package cz.mtulek.trecker.dto.stats;

public record ActivityDataPoint(
    int year,
    int month,
    long count
) {}
