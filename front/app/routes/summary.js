import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

import defaultTheme from '../hs-themes/dark-green-theme';

export default Ember.Route.extend({
  model: function() {
    var summary_url = ENV.apiBaseURL + 'summary';
    return Ember.$.getJSON(summary_url).then(metrics =>{
      var pr_open_per_day_data = metrics.pull_requests.opened_per_day.map(elt => {
        return [new Date(elt[0]).getTime(), elt[1]];
      });

      const model = {
        pr: {
          open_per_day: {
            data: [
              { name: 'PRs Opened Per Day', data: pr_open_per_day_data }
            ],
            mode: 'StockChart',
            opts: { title: { text: 'PRs Opened Per Day' } },
            theme: defaultTheme
          }
        }
      };

      console.log(model);

      return model;
    });
  }
});
