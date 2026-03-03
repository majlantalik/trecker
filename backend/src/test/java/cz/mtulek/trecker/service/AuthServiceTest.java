package cz.mtulek.trecker.service;

import cz.mtulek.trecker.domain.RefreshToken;
import cz.mtulek.trecker.domain.User;
import cz.mtulek.trecker.dto.auth.RegisterRequest;
import cz.mtulek.trecker.repository.RefreshTokenRepository;
import cz.mtulek.trecker.repository.UserRepository;
import cz.mtulek.trecker.security.JwtService;
import jakarta.servlet.http.Cookie;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.security.authentication.AuthenticationManager;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.test.util.ReflectionTestUtils;

import java.time.OffsetDateTime;
import java.util.Arrays;
import java.util.Optional;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class AuthServiceTest {

    @Mock UserRepository userRepository;
    @Mock RefreshTokenRepository refreshTokenRepository;
    @Mock PasswordEncoder passwordEncoder;
    @Mock AuthenticationManager authenticationManager;
    @Mock JwtService jwtService;

    private AuthService authService;

    @BeforeEach
    void setUp() {
        authService = new AuthService(userRepository, refreshTokenRepository, passwordEncoder,
                authenticationManager, jwtService);
        ReflectionTestUtils.setField(authService, "refreshExpirationDays", 30);
        ReflectionTestUtils.setField(authService, "cookieSecure", false);
    }

    @Test
    void register_normalizesEmailToLowercase() {
        RegisterRequest req = new RegisterRequest("USER@EXAMPLE.COM", "password123");
        when(userRepository.existsByEmail("user@example.com")).thenReturn(false);
        when(passwordEncoder.encode(anyString())).thenReturn("hashed");
        when(userRepository.save(any())).thenAnswer(inv -> inv.getArgument(0));

        User result = authService.register(req);

        assertThat(result.getEmail()).isEqualTo("user@example.com");
    }

    @Test
    void register_throwsOnDuplicateEmail() {
        RegisterRequest req = new RegisterRequest("user@example.com", "password123");
        when(userRepository.existsByEmail("user@example.com")).thenReturn(true);

        assertThatThrownBy(() -> authService.register(req))
                .isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void refresh_revokesAllTokensOnReuseAttack() {
        User user = makeUser();
        RefreshToken revoked = makeRefreshToken(user, true, OffsetDateTime.now().plusDays(1));

        String plainToken = "someRandomToken";
        String hash = sha256Hex(plainToken);
        when(refreshTokenRepository.findByTokenHash(hash)).thenReturn(Optional.of(revoked));

        assertThatThrownBy(() -> authService.refresh(plainToken))
                .isInstanceOf(IllegalArgumentException.class);
        verify(refreshTokenRepository).revokeAllByUser(user);
    }

    @Test
    void refresh_throwsOnExpiredToken() {
        User user = makeUser();
        RefreshToken expired = makeRefreshToken(user, false, OffsetDateTime.now().minusDays(1));

        String plainToken = "expiredToken";
        String hash = sha256Hex(plainToken);
        when(refreshTokenRepository.findByTokenHash(hash)).thenReturn(Optional.of(expired));

        assertThatThrownBy(() -> authService.refresh(plainToken))
                .isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void refresh_rotatesTokenPairOnValidToken() {
        User user = makeUser();
        RefreshToken valid = makeRefreshToken(user, false, OffsetDateTime.now().plusDays(10));

        String plainToken = "validToken";
        String hash = sha256Hex(plainToken);
        when(refreshTokenRepository.findByTokenHash(hash)).thenReturn(Optional.of(valid));
        when(jwtService.generateAccessToken(any(), any())).thenReturn("new-access-token");
        when(refreshTokenRepository.save(any())).thenAnswer(inv -> inv.getArgument(0));

        Cookie[] cookies = authService.refresh(plainToken);

        assertThat(valid.isRevoked()).isTrue();
        assertThat(cookies).hasSize(2);
        assertThat(Arrays.stream(cookies).map(Cookie::getName).toList())
                .containsExactlyInAnyOrder("access_token", "refresh_token");
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    private User makeUser() {
        User u = new User();
        u.setId(UUID.randomUUID());
        u.setEmail("user@example.com");
        u.setPasswordHash("hashed");
        return u;
    }

    private RefreshToken makeRefreshToken(User user, boolean revoked, OffsetDateTime expiresAt) {
        RefreshToken t = new RefreshToken();
        t.setUser(user);
        t.setTokenHash("somehash");
        t.setExpiresAt(expiresAt);
        t.setRevoked(revoked);
        return t;
    }

    private String sha256Hex(String input) {
        try {
            java.security.MessageDigest digest = java.security.MessageDigest.getInstance("SHA-256");
            byte[] hash = digest.digest(input.getBytes());
            return java.util.HexFormat.of().formatHex(hash);
        } catch (java.security.NoSuchAlgorithmException e) {
            throw new IllegalStateException(e);
        }
    }
}
