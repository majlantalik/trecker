package cz.mtulek.trecker.security;

import io.jsonwebtoken.Claims;
import io.jsonwebtoken.JwtException;
import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletException;
import jakarta.servlet.http.Cookie;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.security.authentication.UsernamePasswordAuthenticationToken;
import org.springframework.security.core.context.SecurityContextHolder;
import org.springframework.security.core.userdetails.UserDetails;
import org.springframework.security.web.authentication.WebAuthenticationDetailsSource;
import org.springframework.stereotype.Component;
import org.springframework.web.filter.OncePerRequestFilter;

import java.io.IOException;
import java.util.Arrays;

@Component
@RequiredArgsConstructor
@Slf4j
public class JwtAuthFilter extends OncePerRequestFilter {

    private final JwtService jwtService;
    private final UserDetailsServiceImpl userDetailsService;

    @Override
    protected boolean shouldNotFilter(HttpServletRequest request) {
        String path = request.getServletPath();
        return path.equals("/api/auth/login")
            || path.equals("/api/auth/register")
            || path.equals("/api/auth/logout")
            || path.equals("/api/auth/refresh");
    }

    @Override
    protected void doFilterInternal(HttpServletRequest request, HttpServletResponse response, FilterChain filterChain)
        throws ServletException, IOException {

        Cookie[] allCookies = request.getCookies();
        log.debug("JwtAuthFilter: {} cookie(s) on {}: {}", allCookies == null ? 0 : allCookies.length,
            request.getServletPath(),
            allCookies == null ? "none" : Arrays.stream(allCookies).map(Cookie::getName).toList());

        String token = extractCookie(request, "access_token");
        if (token != null) {
            try {
                Claims claims = jwtService.validateAndExtractClaims(token);
                String email = jwtService.extractEmail(claims);

                if (email != null && SecurityContextHolder.getContext().getAuthentication() == null) {
                    UserDetails userDetails = userDetailsService.loadUserByUsername(email);
                    UsernamePasswordAuthenticationToken auth = new UsernamePasswordAuthenticationToken(
                        userDetails, null, userDetails.getAuthorities()
                    );
                    auth.setDetails(new WebAuthenticationDetailsSource().buildDetails(request));
                    SecurityContextHolder.getContext().setAuthentication(auth);
                }
            } catch (JwtException e) {
                log.debug("Invalid JWT token: {}", e.getMessage());
            }
        }

        filterChain.doFilter(request, response);
    }

    private String extractCookie(HttpServletRequest request, String name) {
        Cookie[] cookies = request.getCookies();
        if (cookies == null) return null;
        return Arrays.stream(cookies)
            .filter(c -> name.equals(c.getName()))
            .map(Cookie::getValue)
            .findFirst()
            .orElse(null);
    }
}
