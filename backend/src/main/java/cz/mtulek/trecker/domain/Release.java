package cz.mtulek.trecker.domain;

import jakarta.persistence.*;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

import java.time.OffsetDateTime;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;
import java.util.UUID;

@Entity
@Table(name = "releases")
@Getter
@Setter
@NoArgsConstructor
public class Release {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    private UUID id;

    @Column(nullable = false, length = 500)
    private String artist;

    @Column(nullable = false, length = 500)
    private String title;

    @Column(name = "release_year")
    private Integer releaseYear;

    @Column(name = "album_art_url", length = 2000)
    private String albumArtUrl;

    @Column(length = 100)
    private String country;

    @Column(name = "spotify_id", unique = true, length = 255)
    private String spotifyId;

    @Column(name = "musicbrainz_id", unique = true, length = 36)
    private String musicbrainzId;

    @Column(name = "created_at", nullable = false, updatable = false)
    private OffsetDateTime createdAt = OffsetDateTime.now();

    @ManyToMany(fetch = FetchType.EAGER, cascade = {CascadeType.PERSIST, CascadeType.MERGE})
    @JoinTable(
        name = "release_genres",
        joinColumns = @JoinColumn(name = "release_id"),
        inverseJoinColumns = @JoinColumn(name = "genre_id")
    )
    private Set<Genre> genres = new HashSet<>();

    @ElementCollection(fetch = FetchType.EAGER)
    @CollectionTable(name = "release_streaming_links",
        joinColumns = @JoinColumn(name = "release_id"))
    @MapKeyColumn(name = "service")
    @Column(name = "url", length = 2000)
    private Map<String, String> streamingLinks = new HashMap<>();
}
