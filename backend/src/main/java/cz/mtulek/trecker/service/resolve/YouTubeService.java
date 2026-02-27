package cz.mtulek.trecker.service.resolve;

import cz.mtulek.trecker.dto.ResolvedMetadataDto;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;
import org.springframework.web.reactive.function.client.WebClient;
import reactor.core.publisher.Mono;

import java.util.List;
import java.util.Map;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Service
@Slf4j
public class YouTubeService {

    private final WebClient webClient;

    @Value("${trecker.youtube.api-key:}")
    private String apiKey;

    @Value("${trecker.youtube.api-base}")
    private String apiBase;

    private static final Pattern TITLE_PATTERN = Pattern.compile(
        "^(.+?)\\s*[-–—]\\s*(.+?)(?:\\s*[\\[(].+[\\])])?\\s*$"
    );
    private static final Pattern SUFFIX_PATTERN = Pattern.compile(
        "(?i)\\s*[\\[(](full album|official audio|official video|lyric video|audio|video|hd|4k)[\\])]\\s*"
    );

    public YouTubeService(WebClient.Builder webClientBuilder) {
        this.webClient = webClientBuilder.build();
    }

    public Mono<ResolvedMetadataDto> parseVideoTitle(String videoId) {
        if (apiKey == null || apiKey.isBlank()) {
            log.debug("YouTube API key not configured — skipping parseVideoTitle");
            return Mono.empty();
        }
        log.debug("YouTube: fetching snippet for video ID: {}", videoId);

        return webClient.get()
            .uri(apiBase + "/videos?part=snippet&id={id}&key={key}", videoId, apiKey)
            .retrieve()
            .bodyToMono(String.class)
            .flatMap(response -> {
                log.debug("YouTube raw response: {}", response);
                return Mono.<ResolvedMetadataDto>empty();
            })
            .onErrorResume(e -> {
                log.warn("YouTube API error: {}", e.getMessage());
                return Mono.<ResolvedMetadataDto>empty();
            });
    }

    public ResolvedMetadataDto parseTitleString(String rawTitle) {
        log.debug("YouTube: parsing title string: '{}'", rawTitle);
        String cleaned = SUFFIX_PATTERN.matcher(rawTitle).replaceAll("");
        Matcher m = TITLE_PATTERN.matcher(cleaned.trim());
        if (m.matches()) {
            String artist = m.group(1).trim();
            String title = m.group(2).trim();
            log.debug("YouTube: title parsed as artist='{}', title='{}'", artist, title);
            return new ResolvedMetadataDto(artist, title, null, null, null, List.of(), Map.of(), null, null);
        }
        log.debug("YouTube: title did not match artist-title pattern — using raw string as title");
        return new ResolvedMetadataDto(rawTitle, "", null, null, null, List.of(), Map.of(), null, null);
    }

    public String extractVideoId(String url) {
        log.debug("YouTube: extracting video ID from URL: {}", url);
        Pattern pattern = Pattern.compile(
            "(?:youtube\\.com/watch\\?v=|youtu\\.be/)([a-zA-Z0-9_-]{11})"
        );
        Matcher m = pattern.matcher(url);
        if (m.find()) {
            String videoId = m.group(1);
            log.debug("YouTube: extracted video ID: {}", videoId);
            return videoId;
        }
        log.debug("YouTube: no video ID found in URL: {}", url);
        return null;
    }
}
