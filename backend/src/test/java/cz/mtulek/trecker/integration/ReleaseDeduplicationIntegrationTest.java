package cz.mtulek.trecker.integration;

import cz.mtulek.trecker.dto.ReleaseRequest;
import cz.mtulek.trecker.dto.auth.RegisterRequest;
import cz.mtulek.trecker.repository.ReleaseRepository;
import cz.mtulek.trecker.repository.UserReleaseRepository;
import cz.mtulek.trecker.repository.UserRepository;
import cz.mtulek.trecker.service.AuthService;
import cz.mtulek.trecker.service.ReleaseService;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.testcontainers.service.connection.ServiceConnection;
import org.springframework.test.context.ActiveProfiles;
import org.testcontainers.containers.PostgreSQLContainer;
import org.testcontainers.junit.jupiter.Container;
import org.testcontainers.junit.jupiter.Testcontainers;

import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

import static org.assertj.core.api.Assertions.assertThat;

@SpringBootTest
@Testcontainers
@ActiveProfiles("test")
class ReleaseDeduplicationIntegrationTest {

    @Container
    @ServiceConnection
    static PostgreSQLContainer<?> postgres = new PostgreSQLContainer<>("postgres:17-alpine");

    @Autowired ReleaseService releaseService;
    @Autowired AuthService authService;
    @Autowired ReleaseRepository releaseRepository;
    @Autowired UserReleaseRepository userReleaseRepository;
    @Autowired UserRepository userRepository;

    private UUID userId1;
    private UUID userId2;

    @BeforeEach
    void setUp() {
        userReleaseRepository.deleteAll();
        releaseRepository.deleteAll();
        // Create two users
        var user1 = authService.register(new RegisterRequest(
                "dedup1_" + System.currentTimeMillis() + "@test.com", "password123"));
        var user2 = authService.register(new RegisterRequest(
                "dedup2_" + System.currentTimeMillis() + "@test.com", "password123"));
        userId1 = user1.getId();
        userId2 = user2.getId();
    }

    @Test
    void singleUserCreate_createsExactlyOneReleaseRow() {
        String spotifyId = "single_" + UUID.randomUUID();
        ReleaseRequest req = makeRequest(spotifyId);

        releaseService.create(req, userId1);

        assertThat(releaseRepository.findBySpotifyId(spotifyId)).isPresent();
        assertThat(userReleaseRepository.existsByReleaseIdAndUserId(
                releaseRepository.findBySpotifyId(spotifyId).get().getId(), userId1))
                .isTrue();
    }

    @Test
    void twoUsersAddSameRelease_createsOneReleaseRowAndTwoUserReleaseRows() throws Exception {
        String spotifyId = "concurrent_" + UUID.randomUUID();
        ReleaseRequest req = makeRequest(spotifyId);

        CountDownLatch latch = new CountDownLatch(1);
        ExecutorService executor = Executors.newFixedThreadPool(2);

        executor.submit(() -> {
            latch.await();
            releaseService.create(req, userId1);
            return null;
        });
        executor.submit(() -> {
            latch.await();
            releaseService.create(req, userId2);
            return null;
        });

        latch.countDown();
        executor.shutdown();
        executor.awaitTermination(10, java.util.concurrent.TimeUnit.SECONDS);

        // Exactly one catalog release
        long releaseCount = releaseRepository.findAll().stream()
                .filter(r -> spotifyId.equals(r.getSpotifyId()))
                .count();
        assertThat(releaseCount).isEqualTo(1);

        // Exactly two user_releases
        UUID catalogId = releaseRepository.findBySpotifyId(spotifyId).orElseThrow().getId();
        assertThat(userReleaseRepository.existsByReleaseIdAndUserId(catalogId, userId1)).isTrue();
        assertThat(userReleaseRepository.existsByReleaseIdAndUserId(catalogId, userId2)).isTrue();
    }

    @Test
    void create_isIdempotentForSameUserSameRelease() {
        String spotifyId = "idempotent_" + UUID.randomUUID();
        ReleaseRequest req = makeRequest(spotifyId);

        releaseService.create(req, userId1);
        releaseService.create(req, userId1); // second call — should be a no-op

        long releaseCount = releaseRepository.findAll().stream()
                .filter(r -> spotifyId.equals(r.getSpotifyId()))
                .count();
        assertThat(releaseCount).isEqualTo(1);

        UUID catalogId = releaseRepository.findBySpotifyId(spotifyId).orElseThrow().getId();
        long userReleaseCount = userReleaseRepository.findAll().stream()
                .filter(ur -> ur.getRelease().getId().equals(catalogId)
                        && ur.getUser().getId().equals(userId1))
                .count();
        assertThat(userReleaseCount).isEqualTo(1);
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    private ReleaseRequest makeRequest(String spotifyId) {
        return new ReleaseRequest("Test Artist", "Test Album", 2024, null, null, null,
                Map.of(), List.of(), spotifyId, null);
    }
}
