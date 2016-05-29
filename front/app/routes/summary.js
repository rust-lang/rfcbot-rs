import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

function fixTimestamps(data) {
  return data.map(elt => {
    return [elt[0] * 1000, elt[1]];
  });
}

export default Ember.Route.extend({
  model: function() {
    var summary_url = ENV.apiBaseURL + 'summary';
    return Ember.$.getJSON(summary_url)
      .then(metrics => {

        const bors_retries = metrics.pull_requests.bors_retries.map(elt => {
          return {
            pr_number: elt[0],
            num_retries: elt[1]
          };
        });

        // javascript timestamps are awful, and are in milliseconds
        // this is a cheap operation on the frontend, and seems truly dependent
        // on implementation details of the frontend graphing tools
        const prs_open_per_day = fixTimestamps(metrics.pull_requests.opened_per_day);
        const prs_closed_per_day = fixTimestamps(metrics.pull_requests.closed_per_day);
        const prs_merged_per_day = fixTimestamps(metrics.pull_requests.merged_per_day);
        const prs_days_open_b4_close = fixTimestamps(metrics.pull_requests.days_open_before_close);

        var win_buildbot_times = [];
        var mac_buildbot_times = [];
        var linux_buildbot_times = [];
        var misc_buildbot_times = [];
        metrics.buildbots.per_builder_times_mins.forEach(val => {
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

        var win_buildbot_fails = [];
        var mac_buildbot_fails = [];
        var linux_buildbot_fails = [];
        var misc_buildbot_fails = [];
        metrics.buildbots.per_builder_failures.forEach(val => {
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

        const default_opts = {
          chart: {
            height: 250
          },
          navigator: {
            enabled: false
          },
          scrollbar: {
            enabled: false
          },
          rangeSelector: {
            enabled: false
          }
        };

        const model = {
          issues: {
            days_open_current_mean: metrics.issues.current_open_age_days_mean.toFixed(2),
            num_high_priority: metrics.issues.num_open_p_high_issues,
            num_nightly_regress: metrics.issues.num_open_regression_nightly_issues,
            num_beta_regress: metrics.issues.num_open_regression_beta_issues,
            num_stable_regress: metrics.issues.num_open_regression_stable_issues
          },
          linux_buildbots: {
            per_builder_times: {
              data: linux_buildbot_times,
              mode: 'StockChart',
              opts: Object.assign({
                title: {
                  text: 'Times of Successful CI Builds (Linux)'
                }
              }, default_opts)
            },
            per_builder_fails: {
              data: linux_buildbot_fails,
              mode: 'StockChart',
              opts: Object.assign({
                title: {
                  text: 'Number of Failed CI Builds (Linux)'
                }
              }, default_opts)
            }
          },
          windows_buildbots: {
            per_builder_times: {
              data: win_buildbot_times,
              mode: 'StockChart',
              opts: Object.assign({
                title: {
                  text: 'Times of Successful CI Builds (Windows)'
                }
              }, default_opts)
            },
            per_builder_fails: {
              data: win_buildbot_fails,
              mode: 'StockChart',
              opts: Object.assign({
                title: {
                  text: 'Number of Failed CI Builds (Windows)'
                }
              }, default_opts)
            }
          },
          mac_buildbots: {
            per_builder_times: {
              data: mac_buildbot_times,
              mode: 'StockChart',
              opts: Object.assign({
                title: {
                  text: 'Times of Successful CI Builds (Mac)'
                }
              }, default_opts)
            },
            per_builder_fails: {
              data: mac_buildbot_fails,
              mode: 'StockChart',
              opts: Object.assign({
                title: {
                  text: 'Number of Failed CI Builds (Mac)'
                }
              }, default_opts)
            }
          },
          pr: {
            days_open_current_mean: metrics.pull_requests.current_open_age_days_mean.toFixed(2),
            bors_retries_per_pr: bors_retries,
            open_per_day: {
              data: [{
                name: 'PRs Opened Per Day',
                data: prs_open_per_day
              }, {
                name: 'PRs Closed Per Day',
                data: prs_closed_per_day
              }, {
                name: 'PRs Merged Per Day',
                data: prs_merged_per_day
              }],
              mode: 'StockChart',
              opts: Object.assign({
                title: {
                  text: 'PRs Opened/Closed/Merged Per Day'
                }
              }, default_opts)
            },
            days_open_before_close: {
              data: [{
                name: 'PR Days Open Before Closed (by week)',
                data: prs_days_open_b4_close
              }],
              mode: 'StockChart',
              opts: Object.assign({
                title: {
                  text: 'PR Days Open Before Closed'
                }
              }, default_opts)
            }
          }
        };

        return model;
      });
  }
});
