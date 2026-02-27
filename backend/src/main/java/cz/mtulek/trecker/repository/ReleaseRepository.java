package cz.mtulek.trecker.repository;

import cz.mtulek.trecker.domain.Release;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.stereotype.Repository;

import java.util.Optional;
import java.util.UUID;

@Repository
public interface ReleaseRepository extends JpaRepository<Release, UUID> {

    Optional<Release> findBySpotifyId(String spotifyId);

    Optional<Release> findByMusicbrainzId(String musicbrainzId);
}
