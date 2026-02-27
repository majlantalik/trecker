package cz.mtulek.trecker.dto;

import java.util.List;
import java.util.Map;

public record ResolvedMetadataDto(
    String artist,
    String title,
    Integer releaseYear,
    String albumArtUrl,
    String country,
    List<String> genres,
    Map<String, String> streamingLinks,
    String spotifyId,
    String musicbrainzId
) {}
