package cz.mtulek.trecker.specification;

import cz.mtulek.trecker.domain.Genre;
import cz.mtulek.trecker.domain.Release;
import cz.mtulek.trecker.domain.UserRelease;
import cz.mtulek.trecker.dto.ReleaseFilterParams;
import jakarta.persistence.criteria.*;
import org.springframework.data.jpa.domain.Specification;

import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

public class UserReleaseSpecification implements Specification<UserRelease> {

    private final ReleaseFilterParams params;
    private final UUID userId;

    public UserReleaseSpecification(ReleaseFilterParams params, UUID userId) {
        this.params = params;
        this.userId = userId;
    }

    @Override
    public Predicate toPredicate(Root<UserRelease> root, CriteriaQuery<?> query, CriteriaBuilder cb) {
        List<Predicate> predicates = new ArrayList<>();

        // Mandatory data isolation predicate
        predicates.add(cb.equal(root.get("user").get("id"), userId));

        // UserRelease fields
        if (params.status() != null) {
            predicates.add(cb.equal(root.get("status"), params.status()));
        }

        if (params.ratingMin() != null) {
            predicates.add(cb.greaterThanOrEqualTo(root.get("rating"), params.ratingMin()));
        }

        if (params.ratingMax() != null) {
            predicates.add(cb.lessThanOrEqualTo(root.get("rating"), params.ratingMax()));
        }

        if (params.didNotFinish() != null) {
            predicates.add(cb.equal(root.get("didNotFinish"), params.didNotFinish()));
        }

        // Catalog (Release) fields via join
        Join<UserRelease, Release> releaseJoin = root.join("release", JoinType.INNER);

        if (params.country() != null && !params.country().isBlank()) {
            predicates.add(cb.equal(cb.lower(releaseJoin.get("country")), params.country().toLowerCase()));
        }

        if (params.year() != null) {
            predicates.add(cb.equal(releaseJoin.get("releaseYear"), params.year()));
        }

        if (params.search() != null && !params.search().isBlank()) {
            String pattern = "%" + params.search().toLowerCase() + "%";
            Predicate artistMatch = cb.like(cb.lower(releaseJoin.get("artist")), pattern);
            Predicate titleMatch = cb.like(cb.lower(releaseJoin.get("title")), pattern);
            predicates.add(cb.or(artistMatch, titleMatch));
        }

        if (params.genre() != null && !params.genre().isBlank()) {
            Join<Release, Genre> genreJoin = releaseJoin.join("genres", JoinType.INNER);
            predicates.add(cb.equal(cb.lower(genreJoin.get("name")), params.genre().toLowerCase()));
            query.distinct(true);
        }

        return cb.and(predicates.toArray(new Predicate[0]));
    }
}
