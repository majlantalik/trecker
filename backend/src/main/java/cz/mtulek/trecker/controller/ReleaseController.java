package cz.mtulek.trecker.controller;

import cz.mtulek.trecker.domain.ReleaseStatus;
import cz.mtulek.trecker.domain.User;
import cz.mtulek.trecker.dto.*;
import cz.mtulek.trecker.service.ReleaseService;
import cz.mtulek.trecker.service.resolve.MetadataResolverService;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageRequest;
import org.springframework.data.domain.Pageable;
import org.springframework.data.domain.Sort;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.security.core.annotation.AuthenticationPrincipal;
import org.springframework.web.bind.annotation.*;

import java.util.UUID;

@RestController
@RequestMapping("/api/releases")
@RequiredArgsConstructor
public class ReleaseController {

    private final ReleaseService releaseService;
    private final MetadataResolverService metadataResolverService;

    @PostMapping("/resolve")
    public ResponseEntity<ResolvedMetadataDto> resolve(@RequestBody ResolveRequest request) {
        return metadataResolverService.resolve(request.url(), request.query())
            .map(ResponseEntity::ok)
            .blockOptional()
            .orElseGet(() -> ResponseEntity.noContent().build());
    }

    @PostMapping
    @ResponseStatus(HttpStatus.CREATED)
    public ReleaseResponse create(@Valid @RequestBody ReleaseRequest request, @AuthenticationPrincipal User user) {
        return releaseService.create(request, user.getId());
    }

    @GetMapping
    public Page<ReleaseResponse> getAll(
        @RequestParam(required = false) ReleaseStatus status,
        @RequestParam(required = false) String genre,
        @RequestParam(required = false) String country,
        @RequestParam(required = false) Double ratingMin,
        @RequestParam(required = false) Double ratingMax,
        @RequestParam(required = false) Integer year,
        @RequestParam(required = false) Boolean didNotFinish,
        @RequestParam(required = false) String search,
        @RequestParam(defaultValue = "0") int page,
        @RequestParam(defaultValue = "20") int size,
        @RequestParam(defaultValue = "createdAt") String sort,
        @RequestParam(defaultValue = "DESC") String direction,
        @AuthenticationPrincipal User user
    ) {
        ReleaseFilterParams params = new ReleaseFilterParams(
            status, genre, country, ratingMin, ratingMax, year, didNotFinish, search
        );
        Sort.Direction sortDir = Sort.Direction.fromOptionalString(direction).orElse(Sort.Direction.DESC);
        Pageable pageable = PageRequest.of(page, size, Sort.by(sortDir, sort));
        return releaseService.findAll(params, pageable, user.getId());
    }

    @GetMapping("/random")
    public ReleaseResponse getRandom(@AuthenticationPrincipal User user) {
        return releaseService.findRandom(user.getId());
    }

    @GetMapping("/{id}")
    public ReleaseResponse getById(@PathVariable UUID id, @AuthenticationPrincipal User user) {
        return releaseService.findById(id, user.getId());
    }

    @PatchMapping("/{id}")
    public ReleaseResponse update(@PathVariable UUID id, @Valid @RequestBody ReleaseUpdateRequest request,
                                   @AuthenticationPrincipal User user) {
        return releaseService.update(id, request, user.getId());
    }

    @DeleteMapping("/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void delete(@PathVariable UUID id, @AuthenticationPrincipal User user) {
        releaseService.delete(id, user.getId());
    }
}
