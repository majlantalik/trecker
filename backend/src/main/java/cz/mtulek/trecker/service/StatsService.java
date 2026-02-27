package cz.mtulek.trecker.service;

import cz.mtulek.trecker.domain.ReleaseStatus;
import cz.mtulek.trecker.dto.ReleaseResponse;
import cz.mtulek.trecker.dto.stats.ActivityDataPoint;
import cz.mtulek.trecker.dto.stats.BreakdownItem;
import cz.mtulek.trecker.dto.stats.YearEndEntry;
import cz.mtulek.trecker.repository.UserReleaseRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.data.domain.PageRequest;
import org.springframework.data.domain.Sort;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.List;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicInteger;

@Service
@RequiredArgsConstructor
public class StatsService {

    private final UserReleaseRepository userReleaseRepository;

    @Transactional(readOnly = true)
    public List<ActivityDataPoint> getActivityByMonth(UUID userId) {
        return userReleaseRepository.findActivityByMonth(userId).stream()
            .map(row -> new ActivityDataPoint(
                ((Number) row[0]).intValue(),
                ((Number) row[1]).intValue(),
                ((Number) row[2]).longValue()
            ))
            .toList();
    }

    @Transactional(readOnly = true)
    public List<BreakdownItem> getBreakdownByGenre(UUID userId) {
        return userReleaseRepository.findCountByGenre(userId).stream()
            .map(row -> new BreakdownItem((String) row[0], ((Number) row[1]).longValue()))
            .toList();
    }

    @Transactional(readOnly = true)
    public List<BreakdownItem> getBreakdownByCountry(UUID userId) {
        return userReleaseRepository.findCountByCountry(userId).stream()
            .map(row -> new BreakdownItem((String) row[0], ((Number) row[1]).longValue()))
            .toList();
    }

    @Transactional(readOnly = true)
    public List<ReleaseResponse> getTopRated(int limit, UUID userId) {
        var pageable = PageRequest.of(0, limit, Sort.by(Sort.Direction.DESC, "rating", "dateListened"));
        return userReleaseRepository.findAll(
            (root, query, cb) -> cb.and(
                cb.equal(root.get("user").get("id"), userId),
                cb.equal(root.get("status"), ReleaseStatus.LISTENED),
                cb.isNotNull(root.get("rating"))
            ),
            pageable
        ).map(ur -> ReleaseResponse.from(ur.getRelease(), ur)).getContent();
    }

    @Transactional(readOnly = true)
    public List<YearEndEntry> getYearEnd(int year, UUID userId) {
        OffsetDateTime start = OffsetDateTime.of(year, 1, 1, 0, 0, 0, 0, ZoneOffset.UTC);
        OffsetDateTime end = OffsetDateTime.of(year + 1, 1, 1, 0, 0, 0, 0, ZoneOffset.UTC);

        var pageable = PageRequest.of(0, 50, Sort.by(Sort.Direction.DESC, "rating", "dateListened"));
        var results = userReleaseRepository.findAll(
            (root, query, cb) -> cb.and(
                cb.equal(root.get("user").get("id"), userId),
                cb.equal(root.get("status"), ReleaseStatus.LISTENED),
                cb.isNotNull(root.get("rating")),
                cb.greaterThanOrEqualTo(root.get("dateListened"), start),
                cb.lessThan(root.get("dateListened"), end)
            ),
            pageable
        ).map(ur -> ReleaseResponse.from(ur.getRelease(), ur)).getContent();

        AtomicInteger rank = new AtomicInteger(1);
        return results.stream()
            .map(r -> new YearEndEntry(rank.getAndIncrement(), r))
            .toList();
    }
}
