{
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
      default = pkgs.raulyrs;
      description = ''
        Package for rauly.rs discord bot
      '';
    };
    environmentFile = mkOption {
      type = types.path;
      description = ''
        Path containing the Bot's API keys
      '';
    };
  };
  config = mkIf cfg.enable {
    systemd.services.raulyrs = {
      description = "rauly.rs discord bot";
      after = [ "network.target" ];
      wantedBy = ["multi-user.target"];
      path = [pkgs.yt-dlp];
      serviceConfig = {
        Type = "simple";
        User = "raulyrs";
        ExecStart = lib.getExe cfg.package;
        EnvironmentFile = cfg.environmentFile;
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
