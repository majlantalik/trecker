package cz.mtulek.trecker.dto;

import cz.mtulek.trecker.domain.ReleaseStatus;

public record ReleaseFilterParams(
    ReleaseStatus status,
    String genre,
    String country,
    Double ratingMin,
    Double ratingMax,
    Integer year,
    Boolean didNotFinish,
    String search
) {}
