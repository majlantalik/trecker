package cz.mtulek.trecker.controller;

import tools.jackson.databind.ObjectMapper;
import cz.mtulek.trecker.domain.User;
import cz.mtulek.trecker.dto.auth.LoginRequest;
import cz.mtulek.trecker.dto.auth.RegisterRequest;
import cz.mtulek.trecker.repository.UserRepository;
import cz.mtulek.trecker.security.JwtService;
import cz.mtulek.trecker.security.UserDetailsServiceImpl;
import cz.mtulek.trecker.service.AuthService;
import jakarta.servlet.http.Cookie;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.webmvc.test.autoconfigure.WebMvcTest;
import org.springframework.test.context.bean.override.mockito.MockitoBean;
import org.springframework.http.MediaType;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.test.web.servlet.MockMvc;

import java.util.UUID;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.when;
import static org.springframework.security.test.web.servlet.request.SecurityMockMvcRequestPostProcessors.user;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.*;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.*;

@WebMvcTest(AuthController.class)
@ActiveProfiles("test")
class AuthControllerTest {

    @Autowired MockMvc mockMvc;
    @Autowired ObjectMapper objectMapper;

    @MockitoBean AuthService authService;
    @MockitoBean JwtService jwtService;
    @MockitoBean UserDetailsServiceImpl userDetailsService;
    @MockitoBean UserRepository userRepository;

    @Test
    void register_withValidBody_returns201AndSetsCookies() throws Exception {
        User registeredUser = makeUser();
        when(authService.register(any())).thenReturn(registeredUser);
        when(authService.login(any())).thenReturn(new AuthService.LoginResult(
                registeredUser, makeCookies()
        ));

        RegisterRequest req = new RegisterRequest("user@example.com", "password123");

        mockMvc.perform(post("/api/auth/register")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(objectMapper.writeValueAsString(req)))
                .andExpect(status().isCreated());
    }

    @Test
    void login_withValidCredentials_returns200AndSetsCookies() throws Exception {
        User loggedInUser = makeUser();
        when(authService.login(any())).thenReturn(new AuthService.LoginResult(
                loggedInUser, makeCookies()
        ));

        LoginRequest req = new LoginRequest("user@example.com", "password123");

        mockMvc.perform(post("/api/auth/login")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(objectMapper.writeValueAsString(req)))
                .andExpect(status().isOk());
    }

    @Test
    void logout_withAuth_returns204() throws Exception {
        when(authService.logout(any())).thenReturn(makeCookies());

        mockMvc.perform(post("/api/auth/logout")
                        .with(user(makeUser())))
                .andExpect(status().isNoContent());
    }

    @Test
    void me_withoutAuth_returns401() throws Exception {
        mockMvc.perform(get("/api/auth/me"))
                .andExpect(status().isUnauthorized());
    }

    @Test
    void me_withAuth_returns200() throws Exception {
        mockMvc.perform(get("/api/auth/me")
                        .with(user(makeUser())))
                .andExpect(status().isOk())
                .andExpect(jsonPath("$.email").value("user@example.com"));
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    private User makeUser() {
        User u = new User();
        u.setId(UUID.randomUUID());
        u.setEmail("user@example.com");
        u.setPasswordHash("hashed");
        return u;
    }

    private Cookie[] makeCookies() {
        Cookie access = new Cookie("access_token", "tok");
        access.setPath("/api");
        Cookie refresh = new Cookie("refresh_token", "ref");
        refresh.setPath("/api/auth/refresh");
        return new Cookie[]{access, refresh};
    }
}
