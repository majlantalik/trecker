package cz.mtulek.trecker.repository;

import cz.mtulek.trecker.domain.Release;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.stereotype.Repository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

@Repository
public interface ReleaseRepository extends JpaRepository<Release, UUID> {

    Optional<Release> findBySpotifyId(String spotifyId);

    Optional<Release> findByMusicbrainzId(String musicbrainzId);

    @Query("""
        SELECT r FROM Release r
        WHERE LOWER(r.artist) LIKE LOWER(CONCAT('%', :q, '%'))
           OR LOWER(r.title)  LIKE LOWER(CONCAT('%', :q, '%'))
        ORDER BY r.createdAt DESC
        """)
    List<Release> searchCatalog(@Param("q") String q, Pageable pageable);
}
