package cz.mtulek.trecker.service.resolve;

import cz.mtulek.trecker.dto.ResolvedMetadataDto;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.stereotype.Service;
import org.springframework.util.LinkedMultiValueMap;
import org.springframework.util.MultiValueMap;
import org.springframework.web.reactive.function.BodyInserters;
import org.springframework.web.reactive.function.client.WebClient;
import reactor.core.publisher.Mono;

import java.time.Instant;
import java.util.Base64;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicReference;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Service
@Slf4j
public class SpotifyService {

    private final WebClient webClient;

    @Value("${trecker.spotify.client-id:}")
    private String clientId;

    @Value("${trecker.spotify.client-secret:}")
    private String clientSecret;

    @Value("${trecker.spotify.token-url}")
    private String tokenUrl;

    @Value("${trecker.spotify.api-base}")
    private String apiBase;

    private record TokenCache(String token, Instant expiresAt) {}
    private final AtomicReference<TokenCache> tokenCache = new AtomicReference<>();

    private static final Pattern ALBUM_ID_PATTERN = Pattern.compile(
        "spotify\\.com/album/([a-zA-Z0-9]+)"
    );

    public SpotifyService(WebClient.Builder webClientBuilder) {
        this.webClient = webClientBuilder.build();
    }

    private boolean isConfigured() {
        return clientId != null && !clientId.isBlank() && clientSecret != null && !clientSecret.isBlank();
    }

    public String extractAlbumId(String url) {
        Matcher m = ALBUM_ID_PATTERN.matcher(url);
        return m.find() ? m.group(1) : null;
    }

    private Mono<String> getAccessToken() {
        TokenCache cached = tokenCache.get();
        if (cached != null && cached.expiresAt().isAfter(Instant.now().plusSeconds(60))) {
            log.debug("Spotify token cache HIT, expires at {}", cached.expiresAt());
            return Mono.just(cached.token());
        }
        log.debug("Spotify token cache MISS — fetching new access token");

        String credentials = Base64.getEncoder().encodeToString((clientId + ":" + clientSecret).getBytes());
        MultiValueMap<String, String> form = new LinkedMultiValueMap<>();
        form.add("grant_type", "client_credentials");

        return webClient.post()
            .uri(tokenUrl)
            .header(HttpHeaders.AUTHORIZATION, "Basic " + credentials)
            .contentType(MediaType.APPLICATION_FORM_URLENCODED)
            .body(BodyInserters.fromFormData(form))
            .retrieve()
            .bodyToMono(Map.class)
            .map(body -> {
                String token = (String) body.get("access_token");
                int expiresIn = ((Number) body.get("expires_in")).intValue();
                tokenCache.set(new TokenCache(token, Instant.now().plusSeconds(expiresIn)));
                log.debug("Spotify token acquired, expires in {}s", expiresIn);
                return token;
            });
    }

    @SuppressWarnings("unchecked")
    public Mono<ResolvedMetadataDto> resolveAlbumById(String albumId) {
        if (!isConfigured()) {
            log.debug("Spotify not configured (missing client-id or client-secret) — skipping resolveAlbumById");
            return Mono.empty();
        }
        log.debug("Spotify: resolveAlbumById({})", albumId);
        String spotifyUrl = "https://open.spotify.com/album/" + albumId;
        Map<String, String> streamingLinks = Map.of("spotify", spotifyUrl);

        return getAccessToken().flatMap(token ->
            webClient.get()
                .uri(apiBase + "/albums/{id}", albumId)
                .header(HttpHeaders.AUTHORIZATION, "Bearer " + token)
                .retrieve()
                .bodyToMono(Map.class)
                .flatMap(album -> {
                    String artist = extractArtistName(album);
                    String title = (String) album.get("name");
                    Integer year = parseYear((String) album.get("release_date"));
                    String artUrl = extractImageUrl(album);
                    log.debug("Spotify album response: artist='{}', title='{}', year={}, artUrl={}",
                        artist, title, year, artUrl != null ? "(present)" : "(absent)");

                    List<String> genres = (List<String>) album.get("genres");
                    if (genres == null || genres.isEmpty()) {
                        String artistId = extractArtistId(album);
                        log.debug("Album has no genres — fetching from artist ID: {}", artistId);
                        if (artistId != null) {
                            return fetchArtistGenres(token, artistId).map(artistGenres -> {
                                log.debug("Artist genres fetched: {}", artistGenres);
                                return new ResolvedMetadataDto(artist, title, year, artUrl, null, artistGenres, streamingLinks, albumId, null);
                            }).defaultIfEmpty(new ResolvedMetadataDto(artist, title, year, artUrl, null, List.of(), streamingLinks, albumId, null));
                        }
                    } else {
                        log.debug("Album has {} genre(s) from album metadata: {}", genres.size(), genres);
                    }

                    return Mono.just(new ResolvedMetadataDto(
                        artist, title, year, artUrl, null,
                        genres != null ? genres : List.of(),
                        streamingLinks, albumId, null
                    ));
                })
        ).onErrorResume(e -> {
            log.warn("Spotify resolveAlbumById failed: {}", e.getMessage());
            return Mono.empty();
        });
    }

    @SuppressWarnings("unchecked")
    public Mono<ResolvedMetadataDto> searchByQuery(String query) {
        if (!isConfigured()) {
            log.debug("Spotify not configured — skipping searchByQuery");
            return Mono.empty();
        }
        log.debug("Spotify: searchByQuery('{}')", query);
        return getAccessToken().flatMap(token ->
            webClient.get()
                .uri(apiBase + "/search?q={q}&type=album&limit=1", query)
                .header(HttpHeaders.AUTHORIZATION, "Bearer " + token)
                .retrieve()
                .bodyToMono(Map.class)
                .flatMap(body -> {
                    var albums = (Map<?, ?>) body.get("albums");
                    if (albums == null) {
                        log.debug("Spotify search: no 'albums' key in response");
                        return Mono.empty();
                    }
                    var items = (List<?>) albums.get("items");
                    if (items == null || items.isEmpty()) {
                        log.debug("Spotify search: no results for query '{}'", query);
                        return Mono.empty();
                    }
                    var first = (Map<?, ?>) items.get(0);
                    String albumId = (String) first.get("id");
                    log.debug("Spotify search found album ID: {} for query '{}'", albumId, query);
                    return resolveAlbumById(albumId);
                })
        ).onErrorResume(e -> {
            log.warn("Spotify searchByQuery failed: {}", e.getMessage());
            return Mono.empty();
        });
    }

    @SuppressWarnings("unchecked")
    private Mono<List<String>> fetchArtistGenres(String token, String artistId) {
        log.debug("Spotify: fetching genres for artist ID: {}", artistId);
        return webClient.get()
            .uri(apiBase + "/artists/{id}", artistId)
            .header(HttpHeaders.AUTHORIZATION, "Bearer " + token)
            .retrieve()
            .bodyToMono(Map.class)
            .map(artist -> {
                List<String> genres = (List<String>) artist.get("genres");
                if (genres == null || genres.isEmpty()) {
                    log.debug("Spotify: artist {} has no genres", artistId);
                    return List.<String>of();
                }
                log.debug("Spotify: artist {} genres: {}", artistId, genres);
                return genres;
            })
            .onErrorReturn(List.of());
    }

    @SuppressWarnings("unchecked")
    private String extractArtistName(Map<?, ?> album) {
        var artists = (List<?>) album.get("artists");
        if (artists != null && !artists.isEmpty()) {
            var first = (Map<?, ?>) artists.get(0);
            return (String) first.get("name");
        }
        return "Unknown Artist";
    }

    @SuppressWarnings("unchecked")
    private String extractArtistId(Map<?, ?> album) {
        var artists = (List<?>) album.get("artists");
        if (artists != null && !artists.isEmpty()) {
            var first = (Map<?, ?>) artists.get(0);
            return (String) first.get("id");
        }
        return null;
    }

    @SuppressWarnings("unchecked")
    private String extractImageUrl(Map<?, ?> album) {
        var images = (List<?>) album.get("images");
        if (images != null && !images.isEmpty()) {
            var first = (Map<?, ?>) images.get(0);
            return (String) first.get("url");
        }
        return null;
    }

    private Integer parseYear(String releaseDate) {
        if (releaseDate == null || releaseDate.isBlank()) return null;
        try {
            return Integer.parseInt(releaseDate.substring(0, 4));
        } catch (Exception e) {
            return null;
        }
    }
}
