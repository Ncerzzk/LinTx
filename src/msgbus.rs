use std::sync::{Arc, LazyLock};

use morb::{MorbDataType, Publisher, Subscriber, Topic};

use crate::{adc::AdcRawMsg, mixer::MixerOutMsg};

const LATEST_ONLY_QUEUE_SIZE: u16 = 1;

fn create_or_get_topic<T: MorbDataType>(name: &str) -> Arc<Topic<T>> {
    morb::get_or_create_topic(name.to_string(), LATEST_ONLY_QUEUE_SIZE).unwrap()
}

static ADC_RAW_TOPIC: LazyLock<Arc<Topic<AdcRawMsg>>> =
    LazyLock::new(|| create_or_get_topic("adc_raw"));

static MIXER_OUT_TOPIC: LazyLock<Arc<Topic<MixerOutMsg>>> =
    LazyLock::new(|| create_or_get_topic("mixer_out"));

pub fn adc_raw_publisher() -> Publisher<AdcRawMsg> {
    ADC_RAW_TOPIC.create_publisher()
}

pub fn adc_raw_subscriber() -> TopicReader<AdcRawMsg> {
    TopicReader::new(ADC_RAW_TOPIC.clone())
}

pub fn mixer_out_publisher() -> Publisher<MixerOutMsg> {
    MIXER_OUT_TOPIC.create_publisher()
}

pub fn mixer_out_subscriber() -> TopicReader<MixerOutMsg> {
    TopicReader::new(MIXER_OUT_TOPIC.clone())
}

pub struct TopicReader<T: MorbDataType> {
    subscriber: Subscriber<T>,
}

impl<T: MorbDataType> TopicReader<T> {
    pub fn new(topic: Arc<Topic<T>>) -> Self {
        Self {
            subscriber: topic.create_subscriber(),
        }
    }

    pub fn read(&mut self) -> T {
        self.subscriber.read(None).unwrap()
    }

    #[allow(dead_code)]
    pub fn try_read(&mut self) -> Option<T> {
        self.subscriber.check_update_and_copy()
    }
}

impl<T: MorbDataType> Clone for TopicReader<T> {
    fn clone(&self) -> Self {
        Self {
            subscriber: self.subscriber.clone(),
        }
    }
}
