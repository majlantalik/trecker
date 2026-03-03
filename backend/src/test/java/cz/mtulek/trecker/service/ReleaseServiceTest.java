package cz.mtulek.trecker.service;

import cz.mtulek.trecker.domain.Release;
import cz.mtulek.trecker.domain.ReleaseStatus;
import cz.mtulek.trecker.domain.User;
import cz.mtulek.trecker.domain.UserRelease;
import cz.mtulek.trecker.dto.ReleaseRequest;
import cz.mtulek.trecker.dto.ReleaseUpdateRequest;
import cz.mtulek.trecker.exception.ResourceNotFoundException;
import cz.mtulek.trecker.repository.ReleaseRepository;
import cz.mtulek.trecker.repository.UserReleaseRepository;
import cz.mtulek.trecker.repository.UserRepository;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.dao.DataIntegrityViolationException;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class ReleaseServiceTest {

    @Mock ReleaseRepository releaseRepository;
    @Mock UserReleaseRepository userReleaseRepository;
    @Mock UserRepository userRepository;
    @Mock GenreService genreService;

    private ReleaseService releaseService;

    @BeforeEach
    void setUp() {
        releaseService = new ReleaseService(releaseRepository, userReleaseRepository, userRepository, genreService);
    }

    // ── create ────────────────────────────────────────────────────────────────

    @Test
    void create_reusesExistingCatalogReleaseWhenSpotifyIdMatches() {
        UUID userId = UUID.randomUUID();
        Release existing = makeRelease("spotify123");
        ReleaseRequest req = makeRequest("Artist", "Title", "spotify123");

        when(releaseRepository.findBySpotifyId("spotify123")).thenReturn(Optional.of(existing));
        when(userReleaseRepository.existsByReleaseIdAndUserId(existing.getId(), userId)).thenReturn(false);
        when(userReleaseRepository.save(any())).thenAnswer(inv -> inv.getArgument(0));
        when(userRepository.getReferenceById(userId)).thenReturn(makeUser(userId));

        releaseService.create(req, userId);

        verify(releaseRepository, never()).save(any());
    }

    @Test
    void create_isIdempotentWhenUserAlreadyTracking() {
        UUID userId = UUID.randomUUID();
        Release existing = makeRelease("spotify123");
        UserRelease existingUr = makeUserRelease(existing, makeUser(userId), ReleaseStatus.QUEUED);
        ReleaseRequest req = makeRequest("Artist", "Title", "spotify123");

        when(releaseRepository.findBySpotifyId("spotify123")).thenReturn(Optional.of(existing));
        when(userReleaseRepository.existsByReleaseIdAndUserId(existing.getId(), userId)).thenReturn(true);
        when(userReleaseRepository.findByReleaseIdAndUserId(existing.getId(), userId))
                .thenReturn(Optional.of(existingUr));

        releaseService.create(req, userId);

        verify(userReleaseRepository, never()).save(any());
    }

    @Test
    void create_handlesRaceConditionByRetryingLookup() {
        UUID userId = UUID.randomUUID();
        Release concurrentRelease = makeRelease("spotify123");
        ReleaseRequest req = makeRequest("Artist", "Title", "spotify123");

        // First lookup: nothing found
        when(releaseRepository.findBySpotifyId("spotify123"))
                .thenReturn(Optional.empty())
                .thenReturn(Optional.of(concurrentRelease));
        // Save throws constraint violation
        when(releaseRepository.save(any())).thenThrow(new DataIntegrityViolationException("duplicate key"));
        when(userReleaseRepository.existsByReleaseIdAndUserId(concurrentRelease.getId(), userId)).thenReturn(false);
        when(userReleaseRepository.save(any())).thenAnswer(inv -> inv.getArgument(0));
        when(userRepository.getReferenceById(userId)).thenReturn(makeUser(userId));

        // Should not propagate the exception
        releaseService.create(req, userId);

        verify(releaseRepository, times(2)).findBySpotifyId("spotify123");
    }

    @Test
    void create_setsStatusToQueuedByDefault() {
        UUID userId = UUID.randomUUID();
        Release existing = makeRelease("spotify123");
        ReleaseRequest req = makeRequest("Artist", "Title", "spotify123");

        when(releaseRepository.findBySpotifyId("spotify123")).thenReturn(Optional.of(existing));
        when(userReleaseRepository.existsByReleaseIdAndUserId(existing.getId(), userId)).thenReturn(false);
        when(userRepository.getReferenceById(userId)).thenReturn(makeUser(userId));

        var savedCaptor = org.mockito.ArgumentCaptor.forClass(UserRelease.class);
        when(userReleaseRepository.save(savedCaptor.capture())).thenAnswer(inv -> inv.getArgument(0));

        releaseService.create(req, userId);

        assertThat(savedCaptor.getValue().getStatus()).isEqualTo(ReleaseStatus.QUEUED);
    }

    // ── update ────────────────────────────────────────────────────────────────

    @Test
    void update_autoSetsDateListenedWhenStatusChangesToListened() {
        UUID userId = UUID.randomUUID();
        Release release = makeRelease(null);
        UserRelease ur = makeUserRelease(release, makeUser(userId), ReleaseStatus.QUEUED);

        when(userReleaseRepository.findByReleaseIdAndUserId(release.getId(), userId))
                .thenReturn(Optional.of(ur));
        when(releaseRepository.save(any())).thenAnswer(inv -> inv.getArgument(0));
        when(userReleaseRepository.save(any())).thenAnswer(inv -> inv.getArgument(0));

        ReleaseUpdateRequest req = new ReleaseUpdateRequest(
                null, null, null, null, ReleaseStatus.LISTENED, null, null, null, null, null, null, null, null
        );
        releaseService.update(release.getId(), req, userId);

        assertThat(ur.getDateListened()).isNotNull();
    }

    // ── delete ────────────────────────────────────────────────────────────────

    @Test
    void delete_removesUserReleaseButNotCatalogRelease() {
        UUID userId = UUID.randomUUID();
        Release release = makeRelease(null);
        UserRelease ur = makeUserRelease(release, makeUser(userId), ReleaseStatus.QUEUED);

        when(userReleaseRepository.findByReleaseIdAndUserId(release.getId(), userId))
                .thenReturn(Optional.of(ur));

        releaseService.delete(release.getId(), userId);

        verify(userReleaseRepository).delete(ur);
        verify(releaseRepository, never()).delete(any(Release.class));
    }

    // ── findById ──────────────────────────────────────────────────────────────

    @Test
    void findById_throwsWhenAccessedByDifferentUser() {
        UUID userId = UUID.randomUUID();
        UUID releaseId = UUID.randomUUID();

        when(userReleaseRepository.findByReleaseIdAndUserId(releaseId, userId)).thenReturn(Optional.empty());

        assertThatThrownBy(() -> releaseService.findById(releaseId, userId))
                .isInstanceOf(ResourceNotFoundException.class);
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    private Release makeRelease(String spotifyId) {
        Release r = new Release();
        r.setId(UUID.randomUUID());
        r.setArtist("Artist");
        r.setTitle("Title");
        r.setSpotifyId(spotifyId);
        r.setGenres(Set.of());
        return r;
    }

    private User makeUser(UUID id) {
        User u = new User();
        u.setId(id);
        u.setEmail("user@example.com");
        u.setPasswordHash("hashed");
        return u;
    }

    private UserRelease makeUserRelease(Release release, User user, ReleaseStatus status) {
        UserRelease ur = new UserRelease();
        ur.setId(UUID.randomUUID());
        ur.setRelease(release);
        ur.setUser(user);
        ur.setStatus(status);
        return ur;
    }

    private ReleaseRequest makeRequest(String artist, String title, String spotifyId) {
        return new ReleaseRequest(artist, title, null, null, null, null, Map.of(), List.of(), spotifyId, null);
    }
}
