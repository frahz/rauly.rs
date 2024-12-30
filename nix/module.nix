self: {
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  cfg = config.services.raulyrs;
in {
  options.services.raulyrs = {
    enable = mkEnableOption "rauly.rs discord bot";
    package = mkOption {
      type = types.package;
      default = self.packages.${pkgs.system}.raulyrs;
      description = ''
        Package for rauly.rs discord bot
      '';
    };
    environmentFile = mkOption {
      type = types.path;
      description = ''
        Path containing the Bot's API keys.
        The following keys need to be present:
        DISCORD_TOKEN and WORDNIK_API_KEY
      '';
    };
  };
  config = mkIf cfg.enable {
    systemd.services.raulyrs = {
      description = "rauly.rs discord bot";
      after = ["network-online.target"];
      wants = ["network-online.target"];
      wantedBy = ["multi-user.target"];
      path = [pkgs.yt-dlp];
      serviceConfig = {
        Type = "simple";
        User = "raulyrs";
        ExecStart = lib.getExe cfg.package;
        EnvironmentFile = cfg.environmentFile;
        Restart = "on-failure";
      };
    };

    users = {
      users.raulyrs = {
        description = "rauly.rs service user";
        isSystemUser = true;
        group = "raulyrs";
      };
      groups.raulyrs = {};
    };
  };
}
