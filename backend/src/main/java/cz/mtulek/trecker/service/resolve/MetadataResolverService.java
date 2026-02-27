package cz.mtulek.trecker.service.resolve;

import cz.mtulek.trecker.dto.ResolvedMetadataDto;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import reactor.core.publisher.Mono;

import java.time.Duration;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@Service
@RequiredArgsConstructor
@Slf4j
public class MetadataResolverService {

    private final SpotifyService spotifyService;
    private final MusicBrainzService musicBrainzService;
    private final TidalService tidalService;
    private final YouTubeService youTubeService;

    private static final Duration TIMEOUT = Duration.ofSeconds(8);

    public Mono<ResolvedMetadataDto> resolve(String url, String query) {
        log.debug("resolve() called: url='{}', query='{}'", url, query);
        if (url != null && !url.isBlank()) {
            log.debug("Input has URL — routing to URL resolver");
            return resolveByUrl(url.trim()).timeout(TIMEOUT)
                .onErrorResume(e -> {
                    log.warn("Metadata resolution timed out or failed for URL '{}': {}", url, e.getMessage());
                    return Mono.empty();
                });
        } else if (query != null && !query.isBlank()) {
            log.debug("Input has no URL — routing to query resolver");
            return resolveByQuery(query.trim()).timeout(TIMEOUT)
                .onErrorResume(e -> {
                    log.warn("Metadata resolution failed for query '{}': {}", query, e.getMessage());
                    return Mono.empty();
                });
        }
        log.debug("Neither url nor query provided — returning empty");
        return Mono.empty();
    }

    private Mono<ResolvedMetadataDto> resolveByUrl(String url) {
        if (url.contains("spotify.com/album/")) {
            log.debug("URL matched Spotify album pattern → resolveSpotifyUrl");
            return resolveSpotifyUrl(url);
        } else if (url.contains("tidal.com")) {
            log.debug("URL matched Tidal pattern → resolveTidalUrl");
            return resolveTidalUrl(url);
        } else if (url.contains("youtube.com") || url.contains("youtu.be")) {
            log.debug("URL matched YouTube pattern → resolveYouTubeUrl");
            return resolveYouTubeUrl(url);
        } else {
            log.debug("URL '{}' did not match any known platform — falling back to query resolver", url);
            return resolveByQuery(url);
        }
    }

    private Mono<ResolvedMetadataDto> resolveSpotifyUrl(String url) {
        String albumId = spotifyService.extractAlbumId(url);
        if (albumId == null) {
            log.warn("Could not extract Spotify album ID from URL: {}", url);
            return Mono.empty();
        }
        log.debug("Extracted Spotify album ID: {}", albumId);

        ResolvedMetadataDto empty = new ResolvedMetadataDto(null, null, null, null, null, List.of(), Map.of(), null, null);
        MusicBrainzService.MbReleaseInfo emptyMb = new MusicBrainzService.MbReleaseInfo(null, null);

        log.debug("Launching parallel Spotify + MusicBrainz lookup for albumId={}", albumId);
        Mono<ResolvedMetadataDto> spotifyMono = spotifyService.resolveAlbumById(albumId);
        Mono<MusicBrainzService.MbReleaseInfo> mbMono = musicBrainzService.lookupBySpotifyId(albumId);

        return Mono.zip(
            spotifyMono.onErrorReturn(empty),
            mbMono.defaultIfEmpty(emptyMb)
        ).map(tuple -> {
            ResolvedMetadataDto spotify = tuple.getT1();
            MusicBrainzService.MbReleaseInfo mb = tuple.getT2();
            log.debug("Spotify result: artist='{}', title='{}', genres={}", spotify.artist(), spotify.title(), spotify.genres());
            log.debug("MusicBrainz result: country='{}', mbId='{}'", mb.country(), mb.musicbrainzId());
            ResolvedMetadataDto merged = merge(spotify, mb.country(), mb.musicbrainzId());
            log.debug("Merged result: artist='{}', title='{}', country='{}', mbId='{}'",
                merged.artist(), merged.title(), merged.country(), merged.musicbrainzId());
            return merged;
        }).onErrorResume(e -> {
            log.warn("Spotify resolution failed: {}", e.getMessage());
            return Mono.empty();
        });
    }

    private Mono<ResolvedMetadataDto> resolveTidalUrl(String url) {
        log.debug("Resolving Tidal URL: {}", url);
        return tidalService.resolve(url).flatMap(tidal -> {
            log.debug("Tidal resolved: artist='{}', title='{}', year={}",
                tidal.artist(), tidal.title(), tidal.releaseYear());
            if (tidal.artist() != null && !tidal.artist().isBlank()) {
                log.debug("Fetching MusicBrainz country for Tidal artist: '{}'", tidal.artist());
                return musicBrainzService.lookupArtistCountry(tidal.artist())
                    .defaultIfEmpty("")
                    .map(country -> {
                        log.debug("MusicBrainz country for '{}': '{}'", tidal.artist(), country.isBlank() ? "(none)" : country);
                        return merge(tidal, country.isBlank() ? null : country, null);
                    });
            }
            log.debug("Tidal artist is blank — skipping MusicBrainz country lookup");
            return Mono.just(tidal);
        });
    }

    private Mono<ResolvedMetadataDto> resolveYouTubeUrl(String url) {
        String videoId = youTubeService.extractVideoId(url);
        if (videoId == null) {
            log.warn("Could not extract YouTube video ID from URL: {}", url);
            return Mono.empty();
        }
        log.debug("Extracted YouTube video ID: {}", videoId);

        return youTubeService.parseVideoTitle(videoId)
            .flatMap(partial -> {
                log.debug("YouTube parsed title: artist='{}', title='{}'", partial.artist(), partial.title());
                if (partial.artist() != null && !partial.artist().isBlank()) {
                    String searchQuery = partial.artist() + " " + partial.title();
                    log.debug("Falling back to Spotify search with query: '{}'", searchQuery);
                    return spotifyService.searchByQuery(searchQuery)
                        .defaultIfEmpty(partial);
                }
                log.debug("YouTube artist blank — returning partial result without Spotify enrichment");
                return Mono.just(partial);
            });
    }

    private Mono<ResolvedMetadataDto> resolveByQuery(String query) {
        log.debug("resolveByQuery: '{}'", query);
        Mono<ResolvedMetadataDto> spotifyMono = spotifyService.searchByQuery(query);

        return spotifyMono.flatMap(spotify -> {
            log.debug("Spotify search result: artist='{}', title='{}', spotifyId='{}'",
                spotify.artist(), spotify.title(), spotify.spotifyId());
            if (spotify.artist() != null) {
                log.debug("Fetching MusicBrainz country for artist: '{}'", spotify.artist());
                Mono<String> countryMono = musicBrainzService.lookupArtistCountry(spotify.artist());
                return countryMono.defaultIfEmpty("").map(country -> {
                    log.debug("MusicBrainz country for '{}': '{}'", spotify.artist(), country.isBlank() ? "(none)" : country);
                    ResolvedMetadataDto merged = merge(spotify, country.isBlank() ? null : country, null);
                    log.debug("Query resolution final: artist='{}', title='{}', country='{}'",
                        merged.artist(), merged.title(), merged.country());
                    return merged;
                });
            }
            log.debug("Spotify result has no artist — skipping MusicBrainz lookup");
            return Mono.just(spotify);
        }).onErrorResume(e -> {
            log.warn("Query resolution failed for '{}': {}", query, e.getMessage());
            return Mono.empty();
        });
    }

    private ResolvedMetadataDto merge(ResolvedMetadataDto base, String country, String musicbrainzId) {
        Map<String, String> links = new HashMap<>(base.streamingLinks() != null ? base.streamingLinks() : Map.of());
        return new ResolvedMetadataDto(
            base.artist(),
            base.title(),
            base.releaseYear(),
            base.albumArtUrl(),
            country != null ? country : base.country(),
            base.genres(),
            links,
            base.spotifyId(),
            musicbrainzId != null ? musicbrainzId : base.musicbrainzId()
        );
    }
}
