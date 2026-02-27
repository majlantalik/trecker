package cz.mtulek.trecker.service;

import cz.mtulek.trecker.domain.RefreshToken;
import cz.mtulek.trecker.domain.User;
import cz.mtulek.trecker.dto.auth.LoginRequest;
import cz.mtulek.trecker.dto.auth.RegisterRequest;
import cz.mtulek.trecker.repository.RefreshTokenRepository;
import cz.mtulek.trecker.repository.UserRepository;
import cz.mtulek.trecker.security.JwtService;
import jakarta.servlet.http.Cookie;
import lombok.RequiredArgsConstructor;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.security.authentication.AuthenticationManager;
import org.springframework.security.authentication.UsernamePasswordAuthenticationToken;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.security.SecureRandom;
import java.time.OffsetDateTime;
import java.util.Base64;
import java.util.HexFormat;

@Service
@RequiredArgsConstructor
public class AuthService {

    private final UserRepository userRepository;
    private final RefreshTokenRepository refreshTokenRepository;
    private final PasswordEncoder passwordEncoder;
    private final AuthenticationManager authenticationManager;
    private final JwtService jwtService;

    @Value("${trecker.jwt.refresh-expiration-days:30}")
    private int refreshExpirationDays;

    @Value("${trecker.cookie.secure:false}")
    private boolean cookieSecure;

    @Transactional
    public User register(RegisterRequest request) {
        String email = request.email().toLowerCase().trim();
        if (userRepository.existsByEmail(email)) {
            throw new IllegalArgumentException("Registration failed");
        }
        User user = new User();
        user.setEmail(email);
        user.setPasswordHash(passwordEncoder.encode(request.password()));
        return userRepository.save(user);
    }

    public record LoginResult(User user, Cookie[] cookies) {}

    @Transactional
    public LoginResult login(LoginRequest request) {
        authenticationManager.authenticate(
            new UsernamePasswordAuthenticationToken(request.email(), request.password())
        );
        User user = userRepository.findByEmail(request.email())
            .orElseThrow();
        return new LoginResult(user, issueTokenPair(user));
    }

    @Transactional
    public Cookie[] refresh(String plainToken) {
        String hash = sha256Hex(plainToken);
        RefreshToken stored = refreshTokenRepository.findByTokenHash(hash)
            .orElseThrow(() -> {
                return new IllegalArgumentException("Invalid refresh token");
            });

        if (!stored.isValid()) {
            // Possible reuse attack — revoke all tokens for this user
            refreshTokenRepository.revokeAllByUser(stored.getUser());
            throw new IllegalArgumentException("Refresh token expired or revoked");
        }

        stored.setRevoked(true);
        refreshTokenRepository.save(stored);
        return issueTokenPair(stored.getUser());
    }

    @Transactional
    public Cookie[] logout(User user) {
        refreshTokenRepository.revokeAllByUser(user);
        return new Cookie[]{expiredCookie("access_token", "/api"), expiredCookie("refresh_token", "/api/auth/refresh")};
    }

    private Cookie[] issueTokenPair(User user) {
        String accessToken = jwtService.generateAccessToken(user.getId(), user.getEmail());

        byte[] randomBytes = new byte[32];
        new SecureRandom().nextBytes(randomBytes);
        String plainRefreshToken = Base64.getUrlEncoder().withoutPadding().encodeToString(randomBytes);
        String tokenHash = sha256Hex(plainRefreshToken);

        RefreshToken refreshToken = new RefreshToken();
        refreshToken.setUser(user);
        refreshToken.setTokenHash(tokenHash);
        refreshToken.setExpiresAt(OffsetDateTime.now().plusDays(refreshExpirationDays));
        refreshTokenRepository.save(refreshToken);

        return new Cookie[]{
            buildAccessTokenCookie(accessToken),
            buildRefreshTokenCookie(plainRefreshToken)
        };
    }

    private Cookie buildAccessTokenCookie(String token) {
        Cookie cookie = new Cookie("access_token", token);
        cookie.setHttpOnly(true);
        cookie.setSecure(cookieSecure);
        cookie.setPath("/api");
        cookie.setMaxAge(15 * 60);
        cookie.setAttribute("SameSite", "Strict");
        return cookie;
    }

    private Cookie buildRefreshTokenCookie(String token) {
        Cookie cookie = new Cookie("refresh_token", token);
        cookie.setHttpOnly(true);
        cookie.setSecure(cookieSecure);
        cookie.setPath("/api/auth/refresh");
        cookie.setMaxAge(refreshExpirationDays * 24 * 60 * 60);
        cookie.setAttribute("SameSite", "Strict");
        return cookie;
    }

    private Cookie expiredCookie(String name, String path) {
        Cookie cookie = new Cookie(name, "");
        cookie.setHttpOnly(true);
        cookie.setSecure(cookieSecure);
        cookie.setPath(path);
        cookie.setMaxAge(0);
        cookie.setAttribute("SameSite", "Strict");
        return cookie;
    }

    private String sha256Hex(String input) {
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            byte[] hash = digest.digest(input.getBytes());
            return HexFormat.of().formatHex(hash);
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException("SHA-256 not available", e);
        }
    }
}
