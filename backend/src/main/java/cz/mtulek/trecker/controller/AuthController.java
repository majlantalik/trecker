package cz.mtulek.trecker.controller;

import cz.mtulek.trecker.domain.User;
import cz.mtulek.trecker.dto.auth.LoginRequest;
import cz.mtulek.trecker.dto.auth.RegisterRequest;
import cz.mtulek.trecker.dto.auth.UserDto;
import cz.mtulek.trecker.service.AuthService;
import jakarta.servlet.http.Cookie;
import jakarta.servlet.http.HttpServletResponse;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.http.HttpStatus;
import org.springframework.security.core.annotation.AuthenticationPrincipal;
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api/auth")
@RequiredArgsConstructor
public class AuthController {

    private final AuthService authService;

    @PostMapping("/register")
    @ResponseStatus(HttpStatus.CREATED)
    public UserDto register(@Valid @RequestBody RegisterRequest request, HttpServletResponse response) {
        User user = authService.register(request);
        AuthService.LoginResult result = authService.login(new LoginRequest(request.email(), request.password()));
        for (Cookie cookie : result.cookies()) {
            response.addCookie(cookie);
        }
        return UserDto.from(user);
    }

    @PostMapping("/login")
    public UserDto login(@Valid @RequestBody LoginRequest request, HttpServletResponse response) {
        AuthService.LoginResult result = authService.login(request);
        for (Cookie cookie : result.cookies()) {
            response.addCookie(cookie);
        }
        return UserDto.from(result.user());
    }

    @PostMapping("/logout")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void logout(@AuthenticationPrincipal User user, HttpServletResponse response) {
        Cookie[] cookies = authService.logout(user);
        for (Cookie cookie : cookies) {
            response.addCookie(cookie);
        }
    }

    @GetMapping("/me")
    public UserDto me(@AuthenticationPrincipal User user) {
        return UserDto.from(user);
    }

    @PostMapping("/refresh")
    public void refresh(@CookieValue(name = "refresh_token") String refreshToken, HttpServletResponse response) {
        Cookie[] cookies = authService.refresh(refreshToken);
        for (Cookie cookie : cookies) {
            response.addCookie(cookie);
        }
    }
}
