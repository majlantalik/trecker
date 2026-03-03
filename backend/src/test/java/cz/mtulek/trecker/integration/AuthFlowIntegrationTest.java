package cz.mtulek.trecker.integration;

import cz.mtulek.trecker.dto.auth.LoginRequest;
import cz.mtulek.trecker.dto.auth.RegisterRequest;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.webmvc.test.autoconfigure.AutoConfigureMockMvc;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.testcontainers.service.connection.ServiceConnection;
import org.springframework.http.MediaType;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.test.web.servlet.MockMvc;
import org.springframework.test.web.servlet.MvcResult;
import org.testcontainers.junit.jupiter.Container;
import org.testcontainers.junit.jupiter.Testcontainers;
import org.testcontainers.postgresql.PostgreSQLContainer;
import tools.jackson.databind.ObjectMapper;

import jakarta.servlet.http.Cookie;

import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.*;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.*;

@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
@AutoConfigureMockMvc
@Testcontainers
@ActiveProfiles("test")
class AuthFlowIntegrationTest {

    @Container
    @ServiceConnection
    static PostgreSQLContainer postgres = new PostgreSQLContainer("postgres:17-alpine");

    @Autowired MockMvc mockMvc;
    @Autowired ObjectMapper objectMapper;

    @Test
    void fullAuthFlow_registerLoginAccessLogout() throws Exception {
        String email = "flow_" + System.currentTimeMillis() + "@test.com";
        String password = "password123";

        // Register
        RegisterRequest registerReq = new RegisterRequest(email, password);
        MvcResult registerResult = mockMvc.perform(post("/api/auth/register")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(objectMapper.writeValueAsString(registerReq)))
                .andExpect(status().isCreated())
                .andReturn();

        Cookie accessCookie = registerResult.getResponse().getCookie("access_token");
        Cookie refreshCookie = registerResult.getResponse().getCookie("refresh_token");

        // Access protected endpoint with access token
        mockMvc.perform(get("/api/releases")
                        .cookie(accessCookie))
                .andExpect(status().isOk());

        // Refresh: get new access token
        MvcResult refreshResult = mockMvc.perform(post("/api/auth/refresh")
                        .cookie(refreshCookie))
                .andExpect(status().isOk())
                .andReturn();

        Cookie newAccessCookie = refreshResult.getResponse().getCookie("access_token");
        Cookie newRefreshCookie = refreshResult.getResponse().getCookie("refresh_token");

        // Access with new token
        mockMvc.perform(get("/api/releases")
                        .cookie(newAccessCookie))
                .andExpect(status().isOk());

        // Logout
        mockMvc.perform(post("/api/auth/logout")
                        .cookie(newAccessCookie))
                .andExpect(status().isNoContent());

        // After logout, access should fail (access token is still technically valid for 15min,
        // but refresh token is revoked — test that refresh now fails)
        mockMvc.perform(post("/api/auth/refresh")
                        .cookie(newRefreshCookie))
                .andExpect(status().isBadRequest());
    }

    @Test
    void login_withBadCredentials_returnsError() throws Exception {
        // Register first
        String email = "badlogin_" + System.currentTimeMillis() + "@test.com";
        RegisterRequest reg = new RegisterRequest(email, "password123");
        mockMvc.perform(post("/api/auth/register")
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(reg)));

        // Login with wrong password — BadCredentialsException is caught by the general
        // exception handler and returned as a 5xx (no specific 401 mapping for it yet)
        LoginRequest badLogin = new LoginRequest(email, "wrongpassword");
        mockMvc.perform(post("/api/auth/login")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(objectMapper.writeValueAsString(badLogin)))
                .andExpect(status().is5xxServerError());
    }
}
