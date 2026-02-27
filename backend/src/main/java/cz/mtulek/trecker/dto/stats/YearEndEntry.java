package cz.mtulek.trecker.dto.stats;

import cz.mtulek.trecker.dto.ReleaseResponse;

public record YearEndEntry(
    int rank,
    ReleaseResponse release
) {}
