package cz.mtulek.trecker.dto;

import cz.mtulek.trecker.domain.ReleaseStatus;

public record ReleaseFilterParams(
    ReleaseStatus status,
    String genre,
    String country,
    Short ratingMin,
    Short ratingMax,
    Integer year,
    Boolean didNotFinish,
    String search
) {}
