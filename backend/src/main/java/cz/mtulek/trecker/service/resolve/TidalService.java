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
import org.springframework.web.reactive.function.client.WebClientResponseException;
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
public class TidalService {

    private final WebClient webClient;

    @Value("${trecker.tidal.client-id:}")
    private String clientId;

    @Value("${trecker.tidal.client-secret:}")
    private String clientSecret;

    @Value("${trecker.tidal.token-url}")
    private String tokenUrl;

    @Value("${trecker.tidal.api-base}")
    private String apiBase;

    @Value("${trecker.tidal.country-code:US}")
    private String countryCode;

    private record TokenCache(String token, Instant expiresAt) {}
    private final AtomicReference<TokenCache> tokenCache = new AtomicReference<>();

    // Matches: tidal.com/browse/album/123, listen.tidal.com/album/123, tidal.com/album/123
    private static final Pattern ALBUM_ID_PATTERN = Pattern.compile(
        "(?:tidal\\.com/(?:browse/)?album|listen\\.tidal\\.com/album)/(\\d+)"
    );

    private static final String TIDAL_MEDIA_TYPE = "application/vnd.api+json";

    public TidalService(WebClient.Builder webClientBuilder) {
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
            log.debug("Tidal token cache HIT, expires at {}", cached.expiresAt());
            return Mono.just(cached.token());
        }
        log.debug("Tidal token cache MISS — fetching new access token");

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
                log.debug("Tidal token acquired, expires in {}s", expiresIn);
                return token;
            });
    }

    @SuppressWarnings("unchecked")
    public Mono<ResolvedMetadataDto> resolveAlbumById(String albumId) {
        if (!isConfigured()) {
            log.debug("Tidal not configured (missing client-id or client-secret) — skipping resolveAlbumById");
            return Mono.empty();
        }
        log.debug("Tidal: resolveAlbumById({}) countryCode={}", albumId, countryCode);
        String tidalUrl = "https://tidal.com/browse/album/" + albumId;
        Map<String, String> streamingLinks = Map.of("tidal", tidalUrl);

        return getAccessToken().flatMap(token ->
            webClient.get()
                .uri(apiBase + "/albums/{id}?countryCode={country}&include=artists,coverArt",
                    albumId, countryCode)
                .header(HttpHeaders.AUTHORIZATION, "Bearer " + token)
                .header(HttpHeaders.ACCEPT, TIDAL_MEDIA_TYPE)
                .retrieve()
                .bodyToMono(Map.class)
                .flatMap(body -> {
                    var albumData = (Map<?, ?>) body.get("data");
                    if (albumData == null) {
                        log.warn("Tidal returned no data for album ID: {}", albumId);
                        return Mono.empty();
                    }
                    log.debug("Tidal: received album data object");

                    var attrs = (Map<?, ?>) albumData.get("attributes");
                    if (attrs == null) {
                        log.debug("Tidal: album data has no 'attributes'");
                        return Mono.empty();
                    }

                    String title = (String) attrs.get("title");
                    Integer year = parseYear((String) attrs.get("releaseDate"));
                    List<?> included = (List<?>) body.get("included");
                    String artUrl = extractCoverArtUrl(included);
                    String artist = extractArtistName(albumData, included);
                    log.debug("Tidal album resolved: artist='{}', title='{}', year={}, artUrl={}",
                        artist, title, year, artUrl != null ? "(present)" : "(absent)");

                    return Mono.just(new ResolvedMetadataDto(
                        artist, title, year, artUrl, null, List.of(), streamingLinks, null, null
                    ));
                })
        ).onErrorResume(WebClientResponseException.class, e -> {
            log.warn("Tidal resolveAlbumById failed for {}: {} — body: {}", albumId, e.getMessage(), e.getResponseBodyAsString());
            return Mono.empty();
        }).onErrorResume(e -> {
            log.warn("Tidal resolveAlbumById failed for {}: {}", albumId, e.getMessage());
            return Mono.empty();
        });
    }

    public Mono<ResolvedMetadataDto> resolve(String url) {
        log.debug("Tidal: extracting album ID from URL: {}", url);
        String albumId = extractAlbumId(url);
        if (albumId == null) {
            log.warn("Could not extract Tidal album ID from URL: {}", url);
            return Mono.empty();
        }
        log.debug("Tidal: extracted album ID: {}", albumId);
        return resolveAlbumById(albumId);
    }

    @SuppressWarnings("unchecked")
    private String extractArtistName(Map<?, ?> albumData, List<?> included) {
        try {
            var relationships = (Map<?, ?>) albumData.get("relationships");
            if (relationships == null) return null;

            var artistsRel = (Map<?, ?>) relationships.get("artists");
            if (artistsRel == null) return null;

            var relData = (List<?>) artistsRel.get("data");
            if (relData == null || relData.isEmpty()) return null;

            String primaryArtistId = (String) ((Map<?, ?>) relData.get(0)).get("id");
            if (primaryArtistId == null || included == null) return null;

            for (Object inc : included) {
                var incMap = (Map<?, ?>) inc;
                if ("artists".equals(incMap.get("type")) && primaryArtistId.equals(incMap.get("id"))) {
                    var incAttrs = (Map<?, ?>) incMap.get("attributes");
                    if (incAttrs != null) return (String) incAttrs.get("name");
                }
            }
        } catch (Exception e) {
            log.debug("Failed to extract artist from Tidal response: {}", e.getMessage());
        }
        return null;
    }

    @SuppressWarnings("unchecked")
    private String extractCoverArtUrl(List<?> included) {
        if (included == null) { log.debug("Tidal: included is null"); return null; }
        log.debug("Tidal: included types: {}", included.stream()
            .map(inc -> ((Map<?, ?>) inc).get("type")).toList());
        String best = null;
        int bestWidth = 0;
        for (Object inc : included) {
            var incMap = (Map<?, ?>) inc;
            if (!"artworks".equals(incMap.get("type"))) continue;
            var incAttrs = (Map<?, ?>) incMap.get("attributes");
            if (incAttrs == null) continue;
            var files = (List<?>) incAttrs.get("files");
            if (files == null) continue;
            for (Object fileObj : files) {
                var file = (Map<?, ?>) fileObj;
                String href = (String) file.get("href");
                if (href == null) continue;
                var meta = (Map<?, ?>) file.get("meta");
                Object widthObj = meta != null ? meta.get("width") : null;
                int width = widthObj instanceof Number n ? n.intValue() : 0;
                // Pick largest image up to 640; fallback to any
                if (best == null || (bestWidth < 640 && width > bestWidth)) {
                    best = href;
                    bestWidth = width;
                }
            }
        }
        log.debug("Tidal: cover art URL extracted: {}", best != null ? "(present, width=" + bestWidth + ")" : "(absent)");
        return best;
    }

    private Integer parseYear(String releaseDate) {
        if (releaseDate == null || releaseDate.isBlank()) return null;
        try {
            return Integer.parseInt(releaseDate.substring(0, 4));
        } catch (Exception _) {
            return null;
        }
    }
}
