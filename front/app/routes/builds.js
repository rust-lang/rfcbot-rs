import Ember from 'ember';
import fetch from 'fetch';
import ENV from 'rust-dashboard/config/environment';

function fixTimestamps(data) {
  return data.map(elt => {
    return [elt[0] * 1000, elt[1]];
  });
}

function getDisplayName(build) {
  // TODO: Use better names
  if (build.name === "buildbot") {
    return build.env;
  } else if (build.name === "travis") {
    return build.env;
  } else if (build.name === "appveyor") {
    return build.env;
  }
}

export default Ember.Route.extend({
  model() {
    const summary_url = `${ENV.apiBaseURL}builds`;
    return fetch(summary_url)
      .then(response => response.json())
      .then(metrics => {
        const win_build_times = [];
        const mac_build_times = [];
        const linux_build_times = [];

        metrics.per_builder_times.forEach(val => {
          const os = val[0].os;
          const time = {
            name: getDisplayName(val[0]),
            data: fixTimestamps(val[1])
          };

          if (os === "windows") {
            win_build_times.push(time);
          } else if (os === "linux") {
            linux_build_times.push(time);
          } else if (os === "osx") {
            mac_build_times.push(time);
          }
        });

        const win_build_fails = [];
        const mac_build_fails = [];
        const linux_build_fails = [];

        metrics.per_builder_failures.forEach(val => {
          const os = val[0].os;
          const time = {
            name: getDisplayName(val[0]),
            data: fixTimestamps(val[1])
          };

          if (os === "windows") {
            win_build_fails.push(time);
          } else if (os === "linux") {
            linux_build_fails.push(time);
          } else if (os === "osx") {
            mac_build_fails.push(time);
          }
        });

        const model = {
          linux: {
            per_builder_times: linux_build_times,
            per_builder_fails: linux_build_fails
          },
          windows: {
            per_builder_times: win_build_times,
            per_builder_fails: win_build_fails
          },
          mac: {
            per_builder_times: mac_build_times,
            per_builder_fails: mac_build_fails
          },
          recent_failures: metrics.failures_last_day
        };


        return model;
      });
  }
});
