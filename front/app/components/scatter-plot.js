import Ember from 'ember';

export default Ember.Component.extend({
  opts: {
    chart: {
      type: 'scatter',
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
