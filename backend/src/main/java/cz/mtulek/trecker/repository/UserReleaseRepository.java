package cz.mtulek.trecker.repository;

import cz.mtulek.trecker.domain.UserRelease;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.JpaSpecificationExecutor;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.stereotype.Repository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

@Repository
public interface UserReleaseRepository extends JpaRepository<UserRelease, UUID>, JpaSpecificationExecutor<UserRelease> {

    Optional<UserRelease> findByReleaseIdAndUserId(UUID releaseId, UUID userId);

    boolean existsByReleaseIdAndUserId(UUID releaseId, UUID userId);

    @Query(value = "SELECT ur.* FROM user_releases ur WHERE ur.user_id = :userId AND ur.status = 'QUEUED' ORDER BY RANDOM() LIMIT 1",
        nativeQuery = true)
    Optional<UserRelease> findRandomQueuedByUserId(@Param("userId") UUID userId);

    @Query(value = """
        SELECT EXTRACT(YEAR FROM ur.date_listened) AS year,
               EXTRACT(MONTH FROM ur.date_listened) AS month,
               COUNT(*) AS count
        FROM user_releases ur
        WHERE ur.status = 'LISTENED' AND ur.date_listened IS NOT NULL AND ur.user_id = :userId
        GROUP BY year, month
        ORDER BY year, month
        """, nativeQuery = true)
    List<Object[]> findActivityByMonth(@Param("userId") UUID userId);

    @Query(value = """
        SELECT g.name, COUNT(*) as count
        FROM user_releases ur
        JOIN releases r ON ur.release_id = r.id
        JOIN release_genres rg ON r.id = rg.release_id
        JOIN genres g ON rg.genre_id = g.id
        WHERE ur.status = 'LISTENED' AND ur.user_id = :userId
        GROUP BY g.name
        ORDER BY count DESC
        """, nativeQuery = true)
    List<Object[]> findCountByGenre(@Param("userId") UUID userId);

    @Query(value = """
        SELECT r.country, COUNT(*) as count
        FROM user_releases ur
        JOIN releases r ON ur.release_id = r.id
        WHERE ur.status = 'LISTENED' AND r.country IS NOT NULL AND ur.user_id = :userId
        GROUP BY r.country
        ORDER BY count DESC
        """, nativeQuery = true)
    List<Object[]> findCountByCountry(@Param("userId") UUID userId);

}
