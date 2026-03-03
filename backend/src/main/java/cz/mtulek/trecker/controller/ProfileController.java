package cz.mtulek.trecker.controller;

import cz.mtulek.trecker.domain.User;
import cz.mtulek.trecker.dto.profile.ChangePasswordRequest;
import cz.mtulek.trecker.dto.profile.ProfileDto;
import cz.mtulek.trecker.dto.profile.UpdateProfileRequest;
import cz.mtulek.trecker.service.ProfileService;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.http.HttpStatus;
import org.springframework.security.core.annotation.AuthenticationPrincipal;
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api/profile")
@RequiredArgsConstructor
public class ProfileController {

    private final ProfileService profileService;

    @GetMapping
    public ProfileDto getProfile(@AuthenticationPrincipal User user) {
        return profileService.getProfile(user);
    }

    @PatchMapping
    public ProfileDto updateProfile(@Valid @RequestBody UpdateProfileRequest request,
                                    @AuthenticationPrincipal User user) {
        return profileService.updateProfile(user, request);
    }

    @PostMapping("/password")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void changePassword(@Valid @RequestBody ChangePasswordRequest request,
                               @AuthenticationPrincipal User user) {
        profileService.changePassword(user, request);
    }
}
