package cz.mtulek.trecker.service;

import cz.mtulek.trecker.domain.Genre;
import cz.mtulek.trecker.domain.Release;
import cz.mtulek.trecker.domain.ReleaseStatus;
import cz.mtulek.trecker.domain.UserRelease;
import cz.mtulek.trecker.dto.ReleaseFilterParams;
import cz.mtulek.trecker.dto.ReleaseRequest;
import cz.mtulek.trecker.dto.ReleaseResponse;
import cz.mtulek.trecker.dto.ReleaseUpdateRequest;
import cz.mtulek.trecker.dto.ResolvedMetadataDto;
import cz.mtulek.trecker.exception.ResourceNotFoundException;
import cz.mtulek.trecker.repository.ReleaseRepository;
import cz.mtulek.trecker.repository.UserReleaseRepository;
import cz.mtulek.trecker.repository.UserRepository;
import cz.mtulek.trecker.specification.UserReleaseSpecification;
import lombok.RequiredArgsConstructor;
import org.springframework.dao.DataIntegrityViolationException;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageRequest;
import org.springframework.data.domain.Pageable;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.OffsetDateTime;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;

@Service
@RequiredArgsConstructor
public class ReleaseService {

    private final ReleaseRepository releaseRepository;
    private final UserReleaseRepository userReleaseRepository;
    private final UserRepository userRepository;
    private final GenreService genreService;

    @Transactional
    public ReleaseResponse create(ReleaseRequest request, UUID userId) {
        // Find-or-create catalog release
        Release catalogRelease = findExistingCatalogRelease(request)
            .orElseGet(() -> createCatalogRelease(request));

        // Idempotent — return existing user_release if already tracking
        if (userReleaseRepository.existsByReleaseIdAndUserId(catalogRelease.getId(), userId)) {
            UserRelease existing = userReleaseRepository
                .findByReleaseIdAndUserId(catalogRelease.getId(), userId)
                .orElseThrow();
            return ReleaseResponse.from(catalogRelease, existing);
        }

        // Create tracking row
        UserRelease userRelease = new UserRelease();
        userRelease.setRelease(catalogRelease);
        userRelease.setUser(userRepository.getReferenceById(userId));
        userRelease.setStatus(ReleaseStatus.QUEUED);
        userRelease.setDiscoveryLink(request.discoveryLink());

        return ReleaseResponse.from(catalogRelease, userReleaseRepository.save(userRelease));
    }

    private Optional<Release> findExistingCatalogRelease(ReleaseRequest request) {
        if (request.spotifyId() != null && !request.spotifyId().isBlank()) {
            Optional<Release> found = releaseRepository.findBySpotifyId(request.spotifyId());
            if (found.isPresent()) return found;
        }
        if (request.musicbrainzId() != null && !request.musicbrainzId().isBlank()) {
            Optional<Release> found = releaseRepository.findByMusicbrainzId(request.musicbrainzId());
            if (found.isPresent()) return found;
        }
        return Optional.empty();
    }

    private Release createCatalogRelease(ReleaseRequest request) {
        Release release = new Release();
        release.setArtist(request.artist());
        release.setTitle(request.title());
        release.setReleaseYear(request.releaseYear());
        release.setAlbumArtUrl(request.albumArtUrl());
        release.setCountry(request.country());
        release.setSpotifyId(request.spotifyId());
        release.setMusicbrainzId(request.musicbrainzId());

        if (request.streamingLinks() != null && !request.streamingLinks().isEmpty()) {
            release.getStreamingLinks().putAll(request.streamingLinks());
        }

        if (request.genres() != null) {
            release.setGenres(resolveGenres(request.genres()));
        }

        try {
            return releaseRepository.save(release);
        } catch (DataIntegrityViolationException e) {
            // Race condition: another request created the catalog entry concurrently
            return findExistingCatalogRelease(request)
                .orElseThrow(() -> e);
        }
    }

    @Transactional(readOnly = true)
    public Page<ReleaseResponse> findAll(ReleaseFilterParams params, Pageable pageable, UUID userId) {
        UserReleaseSpecification spec = new UserReleaseSpecification(params, userId);
        return userReleaseRepository.findAll(spec, pageable)
            .map(ur -> ReleaseResponse.from(ur.getRelease(), ur));
    }

    @Transactional(readOnly = true)
    public ReleaseResponse findById(UUID releaseId, UUID userId) {
        return userReleaseRepository.findByReleaseIdAndUserId(releaseId, userId)
            .map(ur -> ReleaseResponse.from(ur.getRelease(), ur))
            .orElseThrow(() -> new ResourceNotFoundException("Release not found: " + releaseId));
    }

    @Transactional(readOnly = true)
    public ReleaseResponse findRandom(UUID userId) {
        return userReleaseRepository.findRandomQueuedByUserId(userId)
            .map(ur -> ReleaseResponse.from(ur.getRelease(), ur))
            .orElseThrow(() -> new ResourceNotFoundException("No queued releases found"));
    }

    @Transactional
    public ReleaseResponse update(UUID releaseId, ReleaseUpdateRequest request, UUID userId) {
        UserRelease userRelease = userReleaseRepository.findByReleaseIdAndUserId(releaseId, userId)
            .orElseThrow(() -> new ResourceNotFoundException("Release not found: " + releaseId));

        Release release = userRelease.getRelease();

        // Update catalog fields
        if (request.artist() != null) release.setArtist(request.artist());
        if (request.title() != null) release.setTitle(request.title());
        if (request.releaseYear() != null) release.setReleaseYear(request.releaseYear());
        if (request.albumArtUrl() != null) release.setAlbumArtUrl(request.albumArtUrl());
        if (request.country() != null) release.setCountry(request.country());

        if (request.streamingLinks() != null) {
            release.getStreamingLinks().clear();
            release.getStreamingLinks().putAll(request.streamingLinks());
        }

        if (request.genres() != null) {
            release.setGenres(resolveGenres(request.genres()));
        }

        releaseRepository.save(release);

        // Update tracking fields
        if (request.discoveryLink() != null) userRelease.setDiscoveryLink(request.discoveryLink());
        if (request.rating() != null) userRelease.setRating(request.rating());
        if (request.didNotFinish() != null) userRelease.setDidNotFinish(request.didNotFinish());
        if (request.notes() != null) userRelease.setNotes(request.notes());
        if (request.dateListened() != null) userRelease.setDateListened(request.dateListened());

        if (request.status() != null) {
            userRelease.setStatus(request.status());
            if (request.status() == ReleaseStatus.LISTENED && userRelease.getDateListened() == null) {
                userRelease.setDateListened(OffsetDateTime.now());
            }
        }

        return ReleaseResponse.from(release, userReleaseRepository.save(userRelease));
    }

    @Transactional
    public void delete(UUID releaseId, UUID userId) {
        UserRelease userRelease = userReleaseRepository.findByReleaseIdAndUserId(releaseId, userId)
            .orElseThrow(() -> new ResourceNotFoundException("Release not found: " + releaseId));
        userReleaseRepository.delete(userRelease);
    }

    @Transactional(readOnly = true)
    public List<ResolvedMetadataDto> searchCatalog(String q, int limit) {
        if (q == null || q.isBlank()) return List.of();
        Pageable pageable = PageRequest.of(0, Math.min(limit, 20));
        return releaseRepository.searchCatalog(q.trim(), pageable)
            .stream()
            .map(r -> new ResolvedMetadataDto(
                r.getArtist(),
                r.getTitle(),
                r.getReleaseYear(),
                r.getAlbumArtUrl(),
                r.getCountry(),
                r.getGenres().stream().map(Genre::getName).sorted().toList(),
                r.getStreamingLinks() != null ? Map.copyOf(r.getStreamingLinks()) : Map.of(),
                r.getSpotifyId(),
                r.getMusicbrainzId()
            ))
            .toList();
    }

    private Set<Genre> resolveGenres(List<String> genreNames) {
        Set<Genre> genres = new HashSet<>();
        for (String name : genreNames) {
            if (name != null && !name.isBlank()) {
                genres.add(genreService.findOrCreate(name.trim()));
            }
        }
        return genres;
    }
}
