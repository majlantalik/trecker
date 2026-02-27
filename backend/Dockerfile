# Stage 1: Build
FROM eclipse-temurin:25-jdk-alpine AS builder

WORKDIR /app

# Copy gradle wrapper and build files first (layer cache optimization)
COPY gradlew .
COPY gradle/ gradle/
COPY build.gradle.kts .
COPY settings.gradle.kts .

# Download dependencies (cached if build files unchanged)
RUN chmod +x gradlew && ./gradlew dependencies --no-daemon -q

# Copy source and build
COPY src/ src/
RUN ./gradlew build -x test --no-daemon -q

# Stage 2: Run
FROM eclipse-temurin:25-jre-alpine AS runtime

RUN addgroup -S trecker && adduser -S trecker -G trecker

WORKDIR /app

COPY --from=builder /app/build/libs/*.jar app.jar

RUN chown trecker:trecker app.jar

USER trecker

EXPOSE 8080 8000

ENTRYPOINT ["java", "-agentlib:jdwp=transport=dt_socket,address=*:8000,server=y,suspend=n", "-jar", "app.jar"]
