package cz.mtulek.trecker.dto;

import cz.mtulek.trecker.domain.Release;
import cz.mtulek.trecker.domain.ReleaseStatus;
import cz.mtulek.trecker.domain.UserRelease;

import java.time.OffsetDateTime;
import java.util.List;
import java.util.Map;
import java.util.UUID;

public record ReleaseResponse(
    UUID id,
    String artist,
    String title,
    Integer releaseYear,
    String albumArtUrl,
    ReleaseStatus status,
    String discoveryLink,
    Map<String, String> streamingLinks,
    String country,
    Short rating,
    boolean didNotFinish,
    OffsetDateTime dateListened,
    String notes,
    OffsetDateTime createdAt,
    List<String> genres
) {
    public static ReleaseResponse from(Release r, UserRelease ur) {
        List<String> genreNames = r.getGenres().stream()
            .map(g -> g.getName())
            .sorted()
            .toList();
        return new ReleaseResponse(
            r.getId(),
            r.getArtist(),
            r.getTitle(),
            r.getReleaseYear(),
            r.getAlbumArtUrl(),
            ur.getStatus(),
            ur.getDiscoveryLink(),
            r.getStreamingLinks() != null ? r.getStreamingLinks() : Map.of(),
            r.getCountry(),
            ur.getRating(),
            ur.isDidNotFinish(),
            ur.getDateListened(),
            ur.getNotes(),
            ur.getCreatedAt(),
            genreNames
        );
    }
}
