package cz.mtulek.trecker.service.resolve;

import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;
import org.springframework.web.reactive.function.client.WebClient;
import reactor.core.publisher.Mono;

import java.time.Duration;
import java.util.Map;

@Service
@Slf4j
public class MusicBrainzService {

    private final WebClient webClient;

    @Value("${trecker.musicbrainz.api-base}")
    private String apiBase;

    @Value("${trecker.musicbrainz.user-agent}")
    private String userAgent;

    public record MbReleaseInfo(String country, String musicbrainzId) {}

    public MusicBrainzService(WebClient.Builder webClientBuilder) {
        this.webClient = webClientBuilder.build();
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
            log.debug("MusicBrainz: first release — country='{}', mbId='{}'", country, musicbrainzId);
            if (country != null && !country.isBlank()) {
                return Mono.just(new MbReleaseInfo(country, musicbrainzId));
            }
            if (musicbrainzId != null) {
                log.debug("MusicBrainz: no country on first release, returning mbId only");
                return Mono.just(new MbReleaseInfo(null, musicbrainzId));
            }
            log.debug("MusicBrainz: first release has neither country nor mbId");
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
