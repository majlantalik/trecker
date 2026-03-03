package cz.mtulek.trecker.specification;

import cz.mtulek.trecker.PostgresRepositoryTestBase;
import cz.mtulek.trecker.domain.*;
import cz.mtulek.trecker.dto.ReleaseFilterParams;
import cz.mtulek.trecker.repository.GenreRepository;
import cz.mtulek.trecker.repository.ReleaseRepository;
import cz.mtulek.trecker.repository.UserReleaseRepository;
import cz.mtulek.trecker.repository.UserRepository;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.jpa.test.autoconfigure.TestEntityManager;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageRequest;

import java.util.List;
import java.util.Set;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;

class UserReleaseSpecificationTest extends PostgresRepositoryTestBase {

    @Autowired UserReleaseRepository userReleaseRepository;
    @Autowired ReleaseRepository releaseRepository;
    @Autowired UserRepository userRepository;
    @Autowired GenreRepository genreRepository;
    @Autowired TestEntityManager em;

    private User user1;
    private User user2;
    private UUID userId1;
    private UUID userId2;

    @BeforeEach
    void setUp() {
        userReleaseRepository.deleteAll();
        releaseRepository.deleteAll();
        userRepository.deleteAll();
        genreRepository.deleteAll();

        user1 = userRepository.save(makeUser("user1@test.com"));
        user2 = userRepository.save(makeUser("user2@test.com"));
        userId1 = user1.getId();
        userId2 = user2.getId();
    }

    @Test
    void spec_filtersOnlyCurrentUserReleases() {
        Release r = persistRelease("Artist A", "Title A", null, null);
        persistUserRelease(r, user1, ReleaseStatus.QUEUED);
        persistUserRelease(r, user2, ReleaseStatus.QUEUED);

        Page<cz.mtulek.trecker.domain.UserRelease> result = query(emptyParams(), userId1);

        assertThat(result.getContent()).hasSize(1);
        assertThat(result.getContent().get(0).getUser().getId()).isEqualTo(userId1);
    }

    @Test
    void spec_filtersByStatus() {
        Release r1 = persistRelease("Artist A", "Album A", null, null);
        Release r2 = persistRelease("Artist B", "Album B", null, null);
        persistUserRelease(r1, user1, ReleaseStatus.QUEUED);
        persistUserRelease(r2, user1, ReleaseStatus.LISTENED);

        ReleaseFilterParams params = new ReleaseFilterParams(ReleaseStatus.QUEUED, null, null, null, null, null, null, null);
        Page<cz.mtulek.trecker.domain.UserRelease> result = query(params, userId1);

        assertThat(result.getContent()).hasSize(1);
        assertThat(result.getContent().get(0).getStatus()).isEqualTo(ReleaseStatus.QUEUED);
    }

    @Test
    void spec_filtersGenreWithoutDuplicates() {
        // Release with 3 genres — must appear exactly once
        Genre g1 = genreRepository.save(new Genre("Jazz"));
        Genre g2 = genreRepository.save(new Genre("Blues"));
        Genre g3 = genreRepository.save(new Genre("Soul"));

        Release r = makeReleaseEntity("Artist X", "Multi-Genre Album", null, null);
        r.setGenres(Set.of(g1, g2, g3));
        r = releaseRepository.save(r);
        persistUserRelease(r, user1, ReleaseStatus.QUEUED);

        ReleaseFilterParams params = new ReleaseFilterParams(null, "Jazz", null, null, null, null, null, null);
        Page<cz.mtulek.trecker.domain.UserRelease> result = query(params, userId1);

        assertThat(result.getTotalElements()).isEqualTo(1);
        assertThat(result.getContent()).hasSize(1);
    }

    @Test
    void spec_caseInsensitiveSearchOnArtistAndTitle() {
        Release r = persistRelease("The Beatles", "Abbey Road", null, null);
        persistUserRelease(r, user1, ReleaseStatus.QUEUED);

        ReleaseFilterParams byArtist = new ReleaseFilterParams(null, null, null, null, null, null, null, "beatles");
        assertThat(query(byArtist, userId1).getTotalElements()).isEqualTo(1);

        ReleaseFilterParams byTitle = new ReleaseFilterParams(null, null, null, null, null, null, null, "ABBEY");
        assertThat(query(byTitle, userId1).getTotalElements()).isEqualTo(1);

        ReleaseFilterParams noMatch = new ReleaseFilterParams(null, null, null, null, null, null, null, "metallica");
        assertThat(query(noMatch, userId1).getTotalElements()).isEqualTo(0);
    }

    @Test
    void spec_caseInsensitiveCountryFilter() {
        Release r = persistRelease("Artist", "Title", "GB", null);
        persistUserRelease(r, user1, ReleaseStatus.QUEUED);

        ReleaseFilterParams upper = new ReleaseFilterParams(null, null, "GB", null, null, null, null, null);
        assertThat(query(upper, userId1).getTotalElements()).isEqualTo(1);

        ReleaseFilterParams lower = new ReleaseFilterParams(null, null, "gb", null, null, null, null, null);
        assertThat(query(lower, userId1).getTotalElements()).isEqualTo(1);
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    private Page<cz.mtulek.trecker.domain.UserRelease> query(ReleaseFilterParams params, UUID userId) {
        return userReleaseRepository.findAll(
                new UserReleaseSpecification(params, userId),
                PageRequest.of(0, 100)
        );
    }

    private ReleaseFilterParams emptyParams() {
        return new ReleaseFilterParams(null, null, null, null, null, null, null, null);
    }

    private Release persistRelease(String artist, String title, String country, String spotifyId) {
        Release r = makeReleaseEntity(artist, title, country, spotifyId);
        return releaseRepository.save(r);
    }

    private Release makeReleaseEntity(String artist, String title, String country, String spotifyId) {
        Release r = new Release();
        r.setArtist(artist);
        r.setTitle(title);
        r.setCountry(country);
        r.setSpotifyId(spotifyId);
        return r;
    }

    private cz.mtulek.trecker.domain.UserRelease persistUserRelease(Release release, User user, ReleaseStatus status) {
        cz.mtulek.trecker.domain.UserRelease ur = new cz.mtulek.trecker.domain.UserRelease();
        ur.setRelease(release);
        ur.setUser(user);
        ur.setStatus(status);
        return userReleaseRepository.save(ur);
    }

    private User makeUser(String email) {
        User u = new User();
        u.setEmail(email);
        u.setPasswordHash("hash");
        return u;
    }
}
