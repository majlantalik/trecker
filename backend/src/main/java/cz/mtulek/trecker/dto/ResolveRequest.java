package cz.mtulek.trecker.dto;

public record ResolveRequest(
    String url,
    String query
) {}
