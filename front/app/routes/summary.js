import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

function linearTrendLine(data) {
  var sum = [0, 0, 0, 0, 0],
    n = 0,
    results = [];

  // exclude the final data point, it will often only be a partial number
  for (; n < data.length - 1; n++) {
    if (data[n][1] != null) {
      sum[0] += data[n][0];
      sum[1] += data[n][1];
      sum[2] += data[n][0] * data[n][0];
      sum[3] += data[n][0] * data[n][1];
      sum[4] += data[n][1] * data[n][1];
    }
  }

  var gradient = (n * sum[3] - sum[0] * sum[1]) / (n * sum[2] - sum[0] * sum[0]);
  var intercept = (sum[1] / n) - (gradient * sum[0]) / n;

  for (var i = 0, len = data.length; i < len; i++) {
    var coordinate = [data[i][0], data[i][0] * gradient + intercept];
    results.push(coordinate);
  }

  return results;
}

export default Ember.Route.extend({
  model: function() {
    var summary_url = ENV.apiBaseURL + 'summary';
    return Ember.$.getJSON(summary_url).then(metrics => {
      const bors_retries = metrics.pull_requests.bors_retries.map(elt => {
        return {
          pr_number: elt[0],
          num_retries: elt[1]
        };
      });
      console.log(bors_retries);
      const model = {
        pr: {
          days_open_current_mean: metrics.pull_requests.current_open_age_days_mean.toFixed(2),
          bors_retries_per_pr: bors_retries,
          open_per_day: {
            data: [{
              name: 'PRs Opened Per Day',
              data: metrics.pull_requests.opened_per_day
            }, {
              name: 'Trend line',
              data: linearTrendLine(metrics.pull_requests.opened_per_day)
            }],
            mode: 'StockChart',
            opts: {
              title: {
                text: 'PRs Opened Per Day'
              }
            }
          },
          closed_per_day: {
            data: [{
              name: 'PRs Closed Per Day',
              data: metrics.pull_requests.closed_per_day
            }, {
              name: 'Trend line',
              data: linearTrendLine(metrics.pull_requests.closed_per_day)
            }],
            mode: 'StockChart',
            opts: {
              title: {
                text: 'PRs Closed Per Day'
              }
            }
          },
          merged_per_day: {
            data: [{
              name: 'PRs Merged Per Day',
              data: metrics.pull_requests.merged_per_day
            }, {
              name: 'Trend line',
              data: linearTrendLine(metrics.pull_requests.merged_per_day)
            }],
            mode: 'StockChart',
            opts: {
              title: {
                text: 'PRs Merged Per Day'
              }
            }
          },
          days_open_before_close: {
            data: [{
              name: 'PR Days Open Before Closed (by week)',
              data: metrics.pull_requests.days_open_before_close
            }, {
              name: 'Days Open Trend line',
              data: linearTrendLine(metrics.pull_requests.days_open_before_close)
            }],
            mode: 'StockChart',
            opts: {
              title: {
                text: 'PR Days Open Before Closed'
              }
            }
          }
        }
      };

      return model;
    });
  }
});
