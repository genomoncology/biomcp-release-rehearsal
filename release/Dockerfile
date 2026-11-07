ARG runtime_image
FROM ${runtime_image}
ARG runtime_packages
ARG debian_snapshot
# The transport initializes platform trust for every mode, including replay.
# The dated snapshot is the only package source, so a pin the live archive has
# retired still installs, and an unavailable snapshot fails the build. It is
# plain http because no CA certificates exist yet; the signed Release file
# carries the trust, and a snapshot's old date must not count as expired.
RUN rm -f /etc/apt/sources.list /etc/apt/sources.list.d/* \
    && echo "deb http://snapshot.debian.org/archive/debian/${debian_snapshot}/ bookworm main" > /etc/apt/sources.list \
    && apt-get -o Acquire::Check-Valid-Until=false update \
    && apt-get install -y --no-install-recommends ${runtime_packages} \
    && rm -rf /var/lib/apt/lists/*
COPY --chmod=0755 biomcp /usr/local/bin/biomcp
COPY LICENSE THIRD-PARTY-NOTICES.txt /usr/share/doc/biomcp/
USER 65532:65532
ENV HOME=/tmp/home XDG_CONFIG_HOME=/tmp/config XDG_CACHE_HOME=/tmp/cache
ENTRYPOINT ["/usr/local/bin/biomcp"]
