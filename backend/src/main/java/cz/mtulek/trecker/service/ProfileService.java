package cz.mtulek.trecker.service;

import cz.mtulek.trecker.domain.User;
import cz.mtulek.trecker.dto.profile.ChangePasswordRequest;
import cz.mtulek.trecker.dto.profile.ProfileDto;
import cz.mtulek.trecker.dto.profile.UpdateProfileRequest;
import cz.mtulek.trecker.repository.UserRepository;
import jakarta.transaction.Transactional;
import lombok.RequiredArgsConstructor;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;

@Service
@RequiredArgsConstructor
public class ProfileService {

    private final UserRepository userRepository;
    private final PasswordEncoder passwordEncoder;

    public ProfileDto getProfile(User user) {
        return ProfileDto.from(user);
    }

    @Transactional
    public ProfileDto updateProfile(User user, UpdateProfileRequest request) {
        user.setDisplayName(request.displayName());
        return ProfileDto.from(userRepository.save(user));
    }

    @Transactional
    public void changePassword(User user, ChangePasswordRequest request) {
        if (!passwordEncoder.matches(request.currentPassword(), user.getPasswordHash())) {
            throw new IllegalArgumentException("Current password is incorrect");
        }
        user.setPasswordHash(passwordEncoder.encode(request.newPassword()));
        userRepository.save(user);
    }
}
