package cz.mtulek.trecker.dto;

import cz.mtulek.trecker.domain.ReleaseStatus;
import jakarta.validation.constraints.DecimalMax;
import jakarta.validation.constraints.DecimalMin;
import jakarta.validation.constraints.Size;

import java.time.OffsetDateTime;
import java.util.List;
import java.util.Map;

public record ReleaseUpdateRequest(
    @Size(max = 500) String artist,
    @Size(max = 500) String title,
    Integer releaseYear,
    @Size(max = 2000) String albumArtUrl,
    ReleaseStatus status,
    @Size(max = 2000) String discoveryLink,
    Map<String, String> streamingLinks,
    String country,
    @DecimalMin("0.5") @DecimalMax("5.0") Double rating,
    Boolean didNotFinish,
    OffsetDateTime dateListened,
    String notes,
    List<String> genres
) {}
