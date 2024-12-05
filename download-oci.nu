#!/usr/bin/env nu

let index = "index.docker.io"
let repository = "library"
let image = "alpine"
let tag = "latest"
let token = http get $"https://auth.docker.io/token?service=registry.docker.io&scope=repository:($repository)/($image):pull" | get token
let manifest = http get --headers [authorization $"Bearer ($token)"] $"https://($index)/v2/($repository)/($image)/manifests/($tag)" | from json

match $manifest.mediaType {
    "application/vnd.oci.image.manifest.v1+json" => {
        echo "Downloading OCI image manifest..."
    },
    "application/vnd.oci.image.index.v1+json" => {
        echo "Downloading OCI image index..."

        let arch_digest = $manifest.manifests | where platform.architecture == "amd64" | get digest.0
        let arch_manifest = http get --headers [authorization $"Bearer ($token)"] $"https://($index)/v2/($repository)/($image)/manifests/($arch_digest)" | from json
        let layers = ($arch_manifest.layers)

        let image_dir = $"/tmp/images/($repository)/($image)/($tag)"
        let layers_dir = $"($image_dir)/layers"

        let rootfs_dir = "/tmp/rootfs"
        mkdir $rootfs_dir

        mkdir $layers_dir
        $arch_manifest | save $"($image_dir)/config.json" -f

        for layer in $layers {
            let digest = $layer.digest
            let url = $"https://($index)/v2/($repository)/($image)/blobs/($digest)"
            let output_file = $"($layers_dir)/($digest).tar"
            echo $"Downloading layer: ($digest)"
            http get --headers [authorization $"Bearer ($token)"] $url | save $output_file -f
            tar xf $output_file -C $rootfs_dir
        }
    },
    _ => {
        echo "Unsupported manifest version: ($manifest.mediaType)"
    }
}
