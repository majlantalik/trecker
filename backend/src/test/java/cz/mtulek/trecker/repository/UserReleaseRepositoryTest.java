package cz.mtulek.trecker.repository;

import cz.mtulek.trecker.PostgresRepositoryTestBase;
import cz.mtulek.trecker.domain.*;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;

import java.util.Optional;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;

class UserReleaseRepositoryTest extends PostgresRepositoryTestBase {

    @Autowired UserReleaseRepository userReleaseRepository;
    @Autowired ReleaseRepository releaseRepository;
    @Autowired UserRepository userRepository;

    private User user;
    private UUID userId;

    @BeforeEach
    void setUp() {
        userReleaseRepository.deleteAll();
        releaseRepository.deleteAll();
        userRepository.deleteAll();

        user = userRepository.save(makeUser("user@test.com"));
        userId = user.getId();
    }

    @Test
    void findRandomQueuedByUserId_returnsOnlyQueuedForUser() {
        Release r1 = persistRelease("Artist A", "Queued Album");
        Release r2 = persistRelease("Artist B", "Listened Album");
        persistUserRelease(r1, user, ReleaseStatus.QUEUED);
        persistUserRelease(r2, user, ReleaseStatus.LISTENED);

        Optional<UserRelease> result = userReleaseRepository.findRandomQueuedByUserId(userId);

        assertThat(result).isPresent();
        assertThat(result.get().getStatus()).isEqualTo(ReleaseStatus.QUEUED);
    }

    @Test
    void findRandomQueuedByUserId_returnsEmptyWhenNoQueuedReleases() {
        Release r = persistRelease("Artist", "Listened Album");
        persistUserRelease(r, user, ReleaseStatus.LISTENED);

        Optional<UserRelease> result = userReleaseRepository.findRandomQueuedByUserId(userId);

        assertThat(result).isEmpty();
    }

    @Test
    void findRandomQueuedByUserId_doesNotReturnOtherUserReleases() {
        User otherUser = userRepository.save(makeUser("other@test.com"));
        Release r = persistRelease("Artist", "Album");
        persistUserRelease(r, otherUser, ReleaseStatus.QUEUED);

        Optional<UserRelease> result = userReleaseRepository.findRandomQueuedByUserId(userId);

        assertThat(result).isEmpty();
    }

    @Test
    void existsByReleaseIdAndUserId_returnsTrueWhenExists() {
        Release r = persistRelease("Artist", "Album");
        persistUserRelease(r, user, ReleaseStatus.QUEUED);

        assertThat(userReleaseRepository.existsByReleaseIdAndUserId(r.getId(), userId)).isTrue();
    }

    @Test
    void existsByReleaseIdAndUserId_returnsFalseWhenNotExists() {
        UUID randomId = UUID.randomUUID();
        assertThat(userReleaseRepository.existsByReleaseIdAndUserId(randomId, userId)).isFalse();
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    private Release persistRelease(String artist, String title) {
        Release r = new Release();
        r.setArtist(artist);
        r.setTitle(title);
        return releaseRepository.save(r);
    }

    private UserRelease persistUserRelease(Release release, User user, ReleaseStatus status) {
        UserRelease ur = new UserRelease();
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
