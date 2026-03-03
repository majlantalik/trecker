package cz.mtulek.trecker.dto.profile;

import jakarta.validation.constraints.Size;

public record UpdateProfileRequest(
    @Size(max = 100) String displayName
) {}
