package cz.mtulek.trecker.domain;

import jakarta.persistence.*;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;

import java.time.OffsetDateTime;
import java.util.UUID;

@Entity
@Table(name = "user_releases",
    uniqueConstraints = @UniqueConstraint(columnNames = {"release_id", "user_id"}))
@Getter
@Setter
@NoArgsConstructor
public class UserRelease {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    private UUID id;

    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "release_id", nullable = false)
    private Release release;

    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "user_id", nullable = false)
    private User user;

    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 20)
    private ReleaseStatus status = ReleaseStatus.QUEUED;

    private Double rating;

    @Column(name = "did_not_finish", nullable = false)
    private boolean didNotFinish = false;

    @Column(name = "date_listened")
    private OffsetDateTime dateListened;

    @Column(columnDefinition = "TEXT")
    private String notes;

    @Column(name = "discovery_link", length = 2000)
    private String discoveryLink;

    @Column(name = "created_at", nullable = false, updatable = false)
    private OffsetDateTime createdAt = OffsetDateTime.now();
}
