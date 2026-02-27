package cz.mtulek.trecker.controller;

import cz.mtulek.trecker.domain.Genre;
import cz.mtulek.trecker.service.GenreService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;

@RestController
@RequestMapping("/api/genres")
@RequiredArgsConstructor
public class GenreController {

    private final GenreService genreService;

    @GetMapping
    public List<String> getAll() {
        return genreService.findAll().stream()
            .map(Genre::getName)
            .sorted()
            .toList();
    }
}
