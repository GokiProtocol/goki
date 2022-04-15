{
  description = "Goki Protocol development environment.";

  inputs = {
    saber-overlay.url = "github:saber-hq/saber-overlay";
  };

  outputs = { self, saber-overlay }: saber-overlay.lib.buildFlakeOutputs {
    setupBuildTools = { pkgs }: {
      anchor = pkgs.anchor-0_24_2;
    };
  };
}
