package cz.mtulek.trecker.service.resolve;

import cz.mtulek.trecker.dto.ResolvedMetadataDto;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import reactor.core.publisher.Mono;
import reactor.test.StepVerifier;

import java.util.List;
import java.util.Map;

import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class MetadataResolverServiceTest {

    @Mock SpotifyService spotifyService;
    @Mock MusicBrainzService musicBrainzService;
    @Mock TidalService tidalService;
    @Mock YouTubeService youTubeService;

    private MetadataResolverService resolver;

    @BeforeEach
    void setUp() {
        resolver = new MetadataResolverService(spotifyService, musicBrainzService, tidalService, youTubeService);
    }

    @Test
    void resolve_nullInputsReturnEmpty() {
        StepVerifier.create(resolver.resolve(null, null))
                .verifyComplete();
    }

    @Test
    void resolve_blankInputsReturnEmpty() {
        StepVerifier.create(resolver.resolve("  ", "  "))
                .verifyComplete();
    }

    @Test
    void resolve_spotifyAlbumUrl_routesToSpotifyAndMergesMbCountry() {
        String url = "https://open.spotify.com/album/abc123";
        ResolvedMetadataDto spotifyResult = makeMetadata("Artist", "Title", null, null);
        MusicBrainzService.MbReleaseInfo mbInfo = new MusicBrainzService.MbReleaseInfo("US", "mb-123");

        when(spotifyService.extractAlbumId(url)).thenReturn("abc123");
        when(spotifyService.resolveAlbumById("abc123")).thenReturn(Mono.just(spotifyResult));
        when(musicBrainzService.lookupBySpotifyId("abc123")).thenReturn(Mono.just(mbInfo));

        StepVerifier.create(resolver.resolve(url, null))
                .assertNext(result -> {
                    assert result.country().equals("US");
                    assert result.musicbrainzId().equals("mb-123");
                })
                .verifyComplete();
    }

    @Test
    void resolve_musicBrainzCountryOverwritesSpotifyCountry() {
        String url = "https://open.spotify.com/album/abc123";
        ResolvedMetadataDto spotifyResult = makeMetadata("Artist", "Title", "GB", null);
        MusicBrainzService.MbReleaseInfo mbInfo = new MusicBrainzService.MbReleaseInfo("US", "mb-123");

        when(spotifyService.extractAlbumId(url)).thenReturn("abc123");
        when(spotifyService.resolveAlbumById("abc123")).thenReturn(Mono.just(spotifyResult));
        when(musicBrainzService.lookupBySpotifyId("abc123")).thenReturn(Mono.just(mbInfo));

        StepVerifier.create(resolver.resolve(url, null))
                .assertNext(result -> {
                    assert result.country().equals("US") : "MB country should overwrite Spotify country";
                })
                .verifyComplete();
    }

    @Test
    void resolve_tidalUrl_routesToTidal() {
        String url = "https://tidal.com/browse/album/12345";
        ResolvedMetadataDto tidalResult = makeMetadata("Artist", "Title", "NO", null);

        when(tidalService.resolve(url)).thenReturn(Mono.just(tidalResult));
        when(musicBrainzService.lookupArtistCountry(any())).thenReturn(Mono.empty());

        StepVerifier.create(resolver.resolve(url, null))
                .assertNext(result -> {
                    assert result.artist().equals("Artist");
                })
                .verifyComplete();

        verify(tidalService).resolve(url);
        verify(spotifyService, never()).resolveAlbumById(any());
    }

    @Test
    void resolve_unknownUrl_fallsBackToQueryResolver() {
        String url = "https://some-unknown-site.com/something";
        ResolvedMetadataDto queryResult = makeMetadata("Artist", "Title", null, null);

        when(spotifyService.searchByQuery(url)).thenReturn(Mono.just(queryResult));
        when(musicBrainzService.lookupArtistCountry(any())).thenReturn(Mono.empty());

        StepVerifier.create(resolver.resolve(url, null))
                .assertNext(result -> {
                    assert result.artist().equals("Artist");
                })
                .verifyComplete();
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    private ResolvedMetadataDto makeMetadata(String artist, String title, String country, String mbId) {
        return new ResolvedMetadataDto(artist, title, null, null, country, List.of(), Map.of(), null, mbId);
    }
}
