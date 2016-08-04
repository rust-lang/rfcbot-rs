import Ember from 'ember';
import fetch from 'fetch';
import ENV from 'rust-dashboard/config/environment';

function fixTimestamps(data) {
  return data.map(elt => {
    return [elt[0] * 1000, elt[1]];
  });
}

export default Ember.Route.extend({
  model: function() {
    const summary_url = ENV.apiBaseURL + 'buildbots';
    return fetch(summary_url)
      .then(metrics => {

        const win_buildbot_times = [];
        const mac_buildbot_times = [];
        const linux_buildbot_times = [];
        const misc_buildbot_times = [];
        metrics.per_builder_times_mins.forEach(val => {
          const time = {
            name: val[0],
            data: fixTimestamps(val[1])
          };

          if (time.name.includes('auto-win')) {
            win_buildbot_times.push(time);
          } else if (time.name.includes('auto-linux')) {
            linux_buildbot_times.push(time);
          } else if (time.name.includes('auto-mac')) {
            mac_buildbot_times.push(time);
          } else {
            misc_buildbot_times.push(time);
          }
        });

        const win_buildbot_fails = [];
        const mac_buildbot_fails = [];
        const linux_buildbot_fails = [];
        const misc_buildbot_fails = [];

        metrics.per_builder_failures.forEach(val => {
          const time = {
            name: val[0],
            data: fixTimestamps(val[1])
          };

          if (time.name.includes('auto-win')) {
            win_buildbot_fails.push(time);
          } else if (time.name.includes('auto-linux')) {
            linux_buildbot_fails.push(time);
          } else if (time.name.includes('auto-mac')) {
            mac_buildbot_fails.push(time);
          } else {
            misc_buildbot_fails.push(time);
          }

        });

        const model = {
          linux_buildbots: {
            per_builder_times: linux_buildbot_times,
            per_builder_fails: linux_buildbot_fails
          },
          windows_buildbots: {
            per_builder_times: win_buildbot_times,
            per_builder_fails: win_buildbot_fails
          },
          mac_buildbots: {
            per_builder_times: mac_buildbot_times,
            per_builder_fails: mac_buildbot_fails
          },
          recent_failures: metrics.failures_last_day
        };

        return model;
      });
  }
});
