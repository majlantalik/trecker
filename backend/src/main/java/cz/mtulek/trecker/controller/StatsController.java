package cz.mtulek.trecker.controller;

import cz.mtulek.trecker.domain.User;
import cz.mtulek.trecker.dto.ReleaseResponse;
import cz.mtulek.trecker.dto.stats.ActivityDataPoint;
import cz.mtulek.trecker.dto.stats.BreakdownItem;
import cz.mtulek.trecker.dto.stats.YearEndEntry;
import cz.mtulek.trecker.service.StatsService;
import lombok.RequiredArgsConstructor;
import org.springframework.security.core.annotation.AuthenticationPrincipal;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

import java.time.Year;
import java.util.List;

@RestController
@RequestMapping("/api/stats")
@RequiredArgsConstructor
public class StatsController {

    private final StatsService statsService;

    @GetMapping("/activity")
    public List<ActivityDataPoint> getActivity(@AuthenticationPrincipal User user) {
        return statsService.getActivityByMonth(user.getId());
    }

    @GetMapping("/by-genre")
    public List<BreakdownItem> getByGenre(@AuthenticationPrincipal User user) {
        return statsService.getBreakdownByGenre(user.getId());
    }

    @GetMapping("/by-country")
    public List<BreakdownItem> getByCountry(@AuthenticationPrincipal User user) {
        return statsService.getBreakdownByCountry(user.getId());
    }

    @GetMapping("/top-rated")
    public List<ReleaseResponse> getTopRated(@RequestParam(defaultValue = "25") int limit,
                                              @AuthenticationPrincipal User user) {
        return statsService.getTopRated(limit, user.getId());
    }

    @GetMapping("/year-end")
    public List<YearEndEntry> getYearEnd(@RequestParam(defaultValue = "0") int year,
                                          @AuthenticationPrincipal User user) {
        int targetYear = year > 0 ? year : Year.now().getValue();
        return statsService.getYearEnd(targetYear, user.getId());
    }
}
