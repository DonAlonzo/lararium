#!/usr/bin/env nu

download_image "https://index.docker.io" "donalonzo" "kodi" "21.1" "amd64" "/tmp/rootfs/kodi"
download_image "https://index.docker.io" "donalonzo" "jellyfin" "10.10.3-r0" "amd64" "/tmp/rootfs/jellyfin"

def download_image [registry, repository, image, tag, arch, destination] {
    mkdir $destination

    let image_dir = $"/tmp/images/($repository)/($image)/($tag)"
    let layers_dir = $"($image_dir)/layers"
    mkdir $layers_dir

    let auth = http get $"($registry)/v2/" -ef | get headers.response | where name == "www-authenticate"
      | get value | split row " " | get 1 | split row "," | split column '=' key value
      | update value { |val| $val.value | str trim --char '"' } | reduce -f {} { |row, acc| $acc | merge { ($row.key): $row.value } }
    let scope = $"repository:($repository)/($image):pull"
    let token = http get $"($auth.realm)?service=($auth.service)&scope=($scope)" | get token
    let manifest = http get --headers [authorization $"Bearer ($token)"] $"($registry)/v2/($repository)/($image)/manifests/($tag)" | from json

    match $manifest.mediaType {
        "application/vnd.oci.image.manifest.v1+json" => {
            echo "Downloading OCI image manifest..."

            $manifest | save $"($image_dir)/config.json" -f
            for layer in $manifest.layers {
                let digest = $layer.digest
                let url = $"($registry)/v2/($repository)/($image)/blobs/($digest)"
                let output_file = $"($layers_dir)/($digest)"
                echo $"Downloading layer: ($digest)"
                http get --headers [authorization $"Bearer ($token)"] $url | save $output_file -f
                tar xf $output_file -C $destination
            }
        },
        "application/vnd.oci.image.index.v1+json" => {
            echo "Downloading OCI image index..."

            let arch_digest = $manifest.manifests | where platform.architecture == $arch | get digest.0
            let manifest = http get --headers [authorization $"Bearer ($token)"] $"($registry)/v2/($repository)/($image)/manifests/($arch_digest)" | from json

            $manifest | save $"($image_dir)/config.json" -f
            for layer in $manifest.layers {
                let digest = $layer.digest
                let url = $"($registry)/v2/($repository)/($image)/blobs/($digest)"
                let output_file = $"($layers_dir)/($digest)"
                echo $"Downloading layer: ($digest)"
                http get --headers [authorization $"Bearer ($token)"] $url | save $output_file -f
                tar xf $output_file -C $destination
            }
        },
        _ => {
            echo "Unsupported manifest version: ($manifest.mediaType)"
        }
    }
}
