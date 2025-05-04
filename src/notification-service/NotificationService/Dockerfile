FROM --platform=$BUILDPLATFORM mcr.microsoft.com/dotnet/sdk:9.0 AS build
ARG TARGETARCH
ARG BUILD_CONFIGURATION=Release
WORKDIR /src

COPY ["NotificationService/NotificationService.csproj", "NotificationService/"]
RUN dotnet restore "NotificationService/NotificationService.csproj" -a $TARGETARCH

COPY . .
WORKDIR "/src/NotificationService"
RUN dotnet build "NotificationService.csproj" -c $BUILD_CONFIGURATION -o /app/build -a $TARGETARCH /p:UseAppHost=false


FROM build AS publish
RUN dotnet publish "NotificationService.csproj" \
    -c $BUILD_CONFIGURATION \
    -o /app/publish \
    -a $TARGETARCH \
    --no-restore

FROM mcr.microsoft.com/dotnet/aspnet:9.0-noble-chiseled
WORKDIR /app

COPY --from=publish /app/publish .

EXPOSE 8080
EXPOSE 8081

ENV DOTNET_EnableDiagnostics=0
ENTRYPOINT ["dotnet", "NotificationService.dll"]
