package cz.mtulek.trecker.security;

import io.jsonwebtoken.Claims;
import io.jsonwebtoken.JwtException;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

class JwtServiceTest {

    private static final String SECRET = "test-secret-that-is-at-least-32-chars-long";
    private JwtService jwtService;

    @BeforeEach
    void setUp() {
        jwtService = new JwtService(SECRET, 900L);
    }

    @Test
    void generateAndValidate_roundTrip() {
        UUID userId = UUID.randomUUID();
        String email = "user@example.com";

        String token = jwtService.generateAccessToken(userId, email);
        Claims claims = jwtService.validateAndExtractClaims(token);

        assertThat(jwtService.extractUserId(claims)).isEqualTo(userId);
        assertThat(jwtService.extractEmail(claims)).isEqualTo(email);
    }

    @Test
    void tamperedToken_throwsJwtException() {
        UUID userId = UUID.randomUUID();
        String token = jwtService.generateAccessToken(userId, "user@example.com");
        String tampered = token.substring(0, token.length() - 4) + "XXXX";

        assertThatThrownBy(() -> jwtService.validateAndExtractClaims(tampered))
                .isInstanceOf(JwtException.class);
    }

    @Test
    void expiredToken_throwsJwtException() {
        JwtService shortLived = new JwtService(SECRET, -1L);
        String token = shortLived.generateAccessToken(UUID.randomUUID(), "user@example.com");

        assertThatThrownBy(() -> jwtService.validateAndExtractClaims(token))
                .isInstanceOf(JwtException.class);
    }

    @Test
    void wrongSigningKey_throwsJwtException() {
        JwtService otherService = new JwtService("different-secret-that-is-at-least-32-chars", 900L);
        String token = otherService.generateAccessToken(UUID.randomUUID(), "user@example.com");

        assertThatThrownBy(() -> jwtService.validateAndExtractClaims(token))
                .isInstanceOf(JwtException.class);
    }
}
