package cz.mtulek.trecker.repository;

import cz.mtulek.trecker.domain.Genre;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.stereotype.Repository;

import java.util.Optional;

@Repository
public interface GenreRepository extends JpaRepository<Genre, Integer> {
    Optional<Genre> findByNameIgnoreCase(String name);
}
