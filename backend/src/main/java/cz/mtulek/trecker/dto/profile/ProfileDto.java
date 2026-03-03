package cz.mtulek.trecker.dto.profile;

import cz.mtulek.trecker.domain.User;

import java.time.OffsetDateTime;
import java.util.UUID;

public record ProfileDto(UUID id, String email, String displayName, OffsetDateTime createdAt) {
    public static ProfileDto from(User user) {
        return new ProfileDto(user.getId(), user.getEmail(), user.getDisplayName(), user.getCreatedAt());
    }
}
