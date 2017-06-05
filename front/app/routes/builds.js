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
  if (build.builder_name === "buildbot") {
    return build.env;
  } else if (build.builder_name === "travis") {
    return build.env;
  } else if (build.builder_name === "appveyor") {
    return build.env;
  }
}

function getURL(build) {
  if (build.builder_name === "buildbot") {
    return `https://buildbot.rust-lang.org/builders/${build.builder_name}/builds/${build.build_id}`;
  } else if (build.builder_name === "travis") {
    return `https://travis-ci.org/rust-lang/rust/jobs/${build.job_id}`;
  } else if (build.builder_name === "appveyor") {
    return `https://ci.appveyor.com/project/rust-lang/rust/build/${build.build_id}/job/${build.job_id}`;
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

        const failures = metrics.failures_last_day.map(build => {
          build.url = getURL(build);
          return build;
        });

        return {
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
          recent_failures: failures,
        };
      });
  }
});
