package cz.mtulek.trecker.service;

import cz.mtulek.trecker.domain.Genre;
import cz.mtulek.trecker.repository.GenreRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;

@Service
@RequiredArgsConstructor
public class GenreService {

    private final GenreRepository genreRepository;

    @Transactional(readOnly = true)
    public List<Genre> findAll() {
        return genreRepository.findAll();
    }

    @Transactional
    public Genre findOrCreate(String name) {
        return genreRepository.findByNameIgnoreCase(name)
            .orElseGet(() -> genreRepository.save(new Genre(name)));
    }
}
