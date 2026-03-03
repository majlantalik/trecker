package cz.mtulek.trecker.controller;

import tools.jackson.databind.ObjectMapper;
import cz.mtulek.trecker.domain.User;
import cz.mtulek.trecker.dto.ReleaseFilterParams;
import cz.mtulek.trecker.dto.ReleaseResponse;
import cz.mtulek.trecker.repository.UserRepository;
import cz.mtulek.trecker.security.JwtService;
import cz.mtulek.trecker.security.UserDetailsServiceImpl;
import cz.mtulek.trecker.service.ReleaseService;
import cz.mtulek.trecker.service.resolve.MetadataResolverService;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.webmvc.test.autoconfigure.WebMvcTest;
import org.springframework.test.context.bean.override.mockito.MockitoBean;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.data.domain.Pageable;
import org.springframework.http.MediaType;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.test.web.servlet.MockMvc;

import java.time.OffsetDateTime;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.springframework.security.test.web.servlet.request.SecurityMockMvcRequestPostProcessors.user;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.*;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.*;

@WebMvcTest(ReleaseController.class)
@ActiveProfiles("test")
class ReleaseControllerTest {

    @Autowired MockMvc mockMvc;
    @Autowired ObjectMapper objectMapper;

    @MockitoBean ReleaseService releaseService;
    @MockitoBean MetadataResolverService metadataResolverService;
    @MockitoBean JwtService jwtService;
    @MockitoBean UserDetailsServiceImpl userDetailsService;
    @MockitoBean UserRepository userRepository;

    @Test
    void getAll_withoutAuth_returns401() throws Exception {
        mockMvc.perform(get("/api/releases"))
                .andExpect(status().isUnauthorized());
    }

    @Test
    void getAll_withAuth_returns200AndCallsService() throws Exception {
        User mockUser = makeUser();
        Page<ReleaseResponse> page = new PageImpl<>(List.of(makeReleaseResponse()));
        when(releaseService.findAll(any(ReleaseFilterParams.class), any(Pageable.class), any(UUID.class)))
                .thenReturn(page);

        mockMvc.perform(get("/api/releases")
                        .with(user(mockUser)))
                .andExpect(status().isOk());

        verify(releaseService).findAll(any(), any(), any());
    }

    @Test
    void create_withInvalidBody_returns400() throws Exception {
        // Missing required 'artist' field
        String body = objectMapper.writeValueAsString(Map.of("title", "Some Album"));

        mockMvc.perform(post("/api/releases")
                        .with(user(makeUser()))
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(body))
                .andExpect(status().isBadRequest());
    }

    @Test
    void delete_withAuth_returns204() throws Exception {
        UUID id = UUID.randomUUID();

        mockMvc.perform(delete("/api/releases/" + id)
                        .with(user(makeUser())))
                .andExpect(status().isNoContent());

        verify(releaseService).delete(any(UUID.class), any(UUID.class));
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    private User makeUser() {
        User u = new User();
        u.setId(UUID.randomUUID());
        u.setEmail("user@example.com");
        u.setPasswordHash("hashed");
        return u;
    }

    private ReleaseResponse makeReleaseResponse() {
        return new ReleaseResponse(
                UUID.randomUUID(), "Artist", "Title", 2024, null,
                cz.mtulek.trecker.domain.ReleaseStatus.QUEUED, null, Map.of(), null, null,
                false, null, null, OffsetDateTime.now(), List.of()
        );
    }
}
