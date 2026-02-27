package cz.mtulek.trecker.dto;

import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Size;

import java.util.List;
import java.util.Map;

public record ReleaseRequest(
    @NotBlank @Size(max = 500) String artist,
    @NotBlank @Size(max = 500) String title,
    Integer releaseYear,
    @Size(max = 2000) String albumArtUrl,
    @Size(max = 100) String country,
    @Size(max = 2000) String discoveryLink,
    Map<String, String> streamingLinks,
    List<String> genres,
    String spotifyId,
    String musicbrainzId
) {}
