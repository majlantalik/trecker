package cz.mtulek.trecker.service.resolve;

import cz.mtulek.trecker.dto.ResolvedMetadataDto;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.http.client.reactive.ReactorClientHttpConnector;
import org.springframework.stereotype.Service;
import org.springframework.web.reactive.function.client.WebClient;
import reactor.core.publisher.Mono;
import reactor.netty.http.client.HttpClient;

import java.time.Duration;
import java.util.List;
import java.util.Map;

@Service
@Slf4j
public class MusicBrainzService {

    private final WebClient webClient;

    @Value("${trecker.musicbrainz.api-base}")
    private String apiBase;

    @Value("${trecker.musicbrainz.user-agent}")
    private String userAgent;

    public record MbReleaseInfo(String country, String musicbrainzId, Integer releaseYear) {}

    public MusicBrainzService(WebClient.Builder webClientBuilder) {
        this.webClient = webClientBuilder
            .clientConnector(new ReactorClientHttpConnector(HttpClient.create().followRedirect(true)))
            .build();
    }

    /**
     * Look up a release by Spotify album ID using MusicBrainz relationships.
     * Returns country and MusicBrainz release ID if found.
     */
    public Mono<MbReleaseInfo> lookupBySpotifyId(String spotifyAlbumId) {
        log.debug("MusicBrainz: lookupBySpotifyId('{}') — waiting 1100ms for rate limit", spotifyAlbumId);
        return Mono.delay(Duration.ofMillis(1100))
            .flatMap(__ -> {
                log.debug("MusicBrainz: querying release by Spotify ID: {}", spotifyAlbumId);
                return webClient.get()
                    .uri(apiBase + "/release?query=reid:{id}&fmt=json", spotifyAlbumId)
                    .header("User-Agent", userAgent)
                    .retrieve()
                    .bodyToMono(Map.class)
                    .flatMap(body -> extractReleaseInfo(body))
                    .onErrorResume(e -> {
                        log.warn("MusicBrainz lookupBySpotifyId failed: {}", e.getMessage());
                        return Mono.empty();
                    });
            });
    }

    /**
     * Search MusicBrainz for a release by query string.
     * If the query contains " - ", splits into artist and release queries.
     * Returns full ResolvedMetadataDto (no art, no genres, no streaming links).
     */
    public Mono<ResolvedMetadataDto> searchRelease(String query) {
        String luceneQuery;
        if (query.contains(" - ")) {
            int sep = query.indexOf(" - ");
            String artist = query.substring(0, sep).trim();
            String release = query.substring(sep + 3).trim();
            luceneQuery = "artist:\"" + artist + "\" AND release:\"" + release + "\"";
        } else {
            luceneQuery = "release:\"" + query + "\"";
        }
        log.debug("MusicBrainz: searchRelease('{}') luceneQuery='{}' — waiting 1100ms for rate limit", query, luceneQuery);
        return Mono.delay(Duration.ofMillis(1100))
            .flatMap(__ -> {
                log.debug("MusicBrainz: querying release search: {}", luceneQuery);
                return webClient.get()
                    .uri(apiBase + "/release?query={q}&fmt=json&limit=1", luceneQuery)
                    .header("User-Agent", userAgent)
                    .retrieve()
                    .bodyToMono(Map.class)
                    .flatMap(body -> extractSearchedReleaseInfo(body))
                    .onErrorResume(e -> {
                        log.warn("MusicBrainz searchRelease failed for '{}': {}", query, e.getMessage());
                        return Mono.empty();
                    });
            });
    }

    /**
     * Look up an artist by name to determine their country.
     */
    public Mono<String> lookupArtistCountry(String artistName) {
        log.debug("MusicBrainz: lookupArtistCountry('{}') — waiting 1100ms for rate limit", artistName);
        return Mono.delay(Duration.ofMillis(1100))
            .flatMap(__ -> {
                log.debug("MusicBrainz: querying artist: '{}'", artistName);
                return webClient.get()
                    .uri(apiBase + "/artist?query={name}&fmt=json&limit=1", artistName)
                    .header("User-Agent", userAgent)
                    .retrieve()
                    .bodyToMono(Map.class)
                    .flatMap(body -> extractCountryFromArtists(body))
                    .onErrorResume(e -> {
                        log.warn("MusicBrainz lookupArtistCountry failed for '{}': {}", artistName, e.getMessage());
                        return Mono.empty();
                    });
            });
    }

    /**
     * Fetch front cover art URL from the Cover Art Archive.
     * Returns a 500px thumbnail URL (or full image as fallback).
     * No rate-limit delay — CAA is a separate service from MusicBrainz API.
     */
    public Mono<String> fetchCoverArt(String mbid) {
        log.debug("CoverArtArchive: fetching cover art for mbId='{}'", mbid);
        return webClient.get()
            .uri("https://coverartarchive.org/release/{mbid}", mbid)
            .header("Accept", "application/json")
            .retrieve()
            .bodyToMono(Map.class)
            .flatMap(this::extractCoverArtUrl)
            .onErrorResume(e -> {
                log.debug("CoverArtArchive: no art for mbId='{}': {}", mbid, e.getMessage());
                return Mono.empty();
            });
    }

    @SuppressWarnings("unchecked")
    private Mono<String> extractCoverArtUrl(Map<?, ?> body) {
        try {
            var images = (List<?>) body.get("images");
            if (images == null || images.isEmpty()) return Mono.empty();

            // Find front image; fall back to first image
            Map<?, ?> chosen = null;
            for (Object img : images) {
                var image = (Map<?, ?>) img;
                if (chosen == null) chosen = image;
                if (Boolean.TRUE.equals(image.get("front"))) { chosen = image; break; }
            }
            if (chosen == null) return Mono.empty();

            var thumbnails = (Map<?, ?>) chosen.get("thumbnails");
            if (thumbnails != null) {
                for (String size : List.of("500", "250", "1200")) {
                    String url = (String) thumbnails.get(size);
                    if (url != null) { log.debug("CoverArtArchive: found {}px thumbnail", size); return Mono.just(url); }
                }
            }
            String url = (String) chosen.get("image");
            return url != null ? Mono.just(url) : Mono.empty();
        } catch (Exception e) {
            log.debug("Failed to parse Cover Art Archive response: {}", e.getMessage());
            return Mono.empty();
        }
    }

    @SuppressWarnings("unchecked")
    private Mono<ResolvedMetadataDto> extractSearchedReleaseInfo(Map<?, ?> body) {
        try {
            var releases = (java.util.List<?>) body.get("releases");
            if (releases == null || releases.isEmpty()) {
                log.debug("MusicBrainz searchRelease: no releases in response");
                return Mono.empty();
            }
            var first = (Map<String, Object>) releases.get(0);
            String mbId = (String) first.get("id");
            String title = (String) first.get("title");
            String country = (String) first.get("country");

            // Parse year from date field (first 4 chars)
            Integer year = null;
            String date = (String) first.get("date");
            if (date != null && date.length() >= 4) {
                try { year = Integer.parseInt(date.substring(0, 4)); } catch (Exception _) {}
            }

            // Extract artist from artist-credit
            String artist = null;
            var artistCredit = (java.util.List<?>) first.get("artist-credit");
            if (artistCredit != null && !artistCredit.isEmpty()) {
                var credit = (Map<?, ?>) artistCredit.get(0);
                artist = (String) credit.get("name");
                if (artist == null) {
                    var artistObj = (Map<?, ?>) credit.get("artist");
                    if (artistObj != null) artist = (String) artistObj.get("name");
                }
            }

            log.debug("MusicBrainz searchRelease result: artist='{}', title='{}', year={}, country='{}', mbId='{}'",
                artist, title, year, country, mbId);

            if (artist == null && title == null) return Mono.empty();

            return Mono.just(new ResolvedMetadataDto(artist, title, year, null, country, List.of(), Map.of(), null, mbId));
        } catch (Exception e) {
            log.debug("Failed to parse MusicBrainz search response: {}", e.getMessage());
            return Mono.empty();
        }
    }

    @SuppressWarnings("unchecked")
    private Mono<MbReleaseInfo> extractReleaseInfo(Map<?, ?> body) {
        try {
            var releases = (java.util.List<?>) body.get("releases");
            if (releases == null || releases.isEmpty()) {
                log.debug("MusicBrainz: no releases in response");
                return Mono.empty();
            }
            log.debug("MusicBrainz: {} release(s) in response", releases.size());
            var first = (Map<String, Object>) releases.get(0);
            String country = (String) first.get("country");
            String musicbrainzId = (String) first.get("id");

            Integer year = null;
            String date = (String) first.get("date");
            if (date != null && date.length() >= 4) {
                try { year = Integer.parseInt(date.substring(0, 4)); } catch (Exception _) {}
            }

            log.debug("MusicBrainz: first release — country='{}', mbId='{}', year={}", country, musicbrainzId, year);
            if (musicbrainzId != null) {
                return Mono.just(new MbReleaseInfo(country, musicbrainzId, year));
            }
            log.debug("MusicBrainz: first release has no mbId");
        } catch (Exception e) {
            log.debug("Failed to parse MusicBrainz release response: {}", e.getMessage());
        }
        return Mono.empty();
    }

    @SuppressWarnings("unchecked")
    private Mono<String> extractCountryFromArtists(Map<?, ?> body) {
        try {
            var artists = (java.util.List<?>) body.get("artists");
            if (artists == null || artists.isEmpty()) {
                log.debug("MusicBrainz: no artists in response");
                return Mono.empty();
            }
            log.debug("MusicBrainz: {} artist(s) in response", artists.size());
            var first = (Map<String, Object>) artists.get(0);
            String country = (String) first.get("country");
            if (country != null && !country.isBlank()) {
                log.debug("MusicBrainz: artist country='{}'", country);
                return Mono.just(country);
            }
            // Try area as fallback
            log.debug("MusicBrainz: no 'country' field on artist — trying 'area' fallback");
            var area = (Map<String, Object>) first.get("area");
            if (area != null) {
                String areaName = (String) area.get("name");
                if (areaName != null && !areaName.isBlank()) {
                    log.debug("MusicBrainz: artist area name='{}'", areaName);
                    return Mono.just(areaName);
                }
            }
            log.debug("MusicBrainz: no country or area found for artist");
        } catch (Exception e) {
            log.debug("Failed to parse MusicBrainz artist response: {}", e.getMessage());
        }
        return Mono.empty();
    }
}
