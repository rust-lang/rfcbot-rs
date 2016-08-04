import Ember from 'ember';

export default Ember.Component.extend({
  opts: {
    chart: {
      height: 350
    },
    navigator: {
      enabled: false
    },
    scrollbar: {
      enabled: false
    },
    rangeSelector: {
      enabled: false
    },
    xAxis: {
      type: 'datetime'
    },
    legend: {
      enabled: false
    },
    title: {
      text: null
    }
  }
});
